use std::io::{Error as IoError, Result as IoResult, Write};
use core::sync::atomic::{AtomicU64, AtomicU8};
use std::fs::File as Handle;
use core::time::Duration;
use core::fmt::Display;
use std::path::PathBuf;
use alloc::sync::Arc;
use std::sync::Mutex;

pub(crate) const CAPACITY: usize = 16;
pub(crate) const LINE: usize     = 2048;

pub(crate) type Sinks = Mutex<Vec<Box<dyn Sink>>>;

/// The severity of an event, lowest first.
#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    /// Every step, for following one request through.
    Trace,
    /// What a developer wants while working on it.
    Debug,
    /// What an operator wants in normal service.
    #[default]
    Info,
    /// Something is wrong but the work went on.
    Warn,
    /// The work failed.
    Error,
    /// The process cannot go on.
    Fatal,
}

/// How a sink renders a record: one line for people, or one JSON object per line for machines.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Shape {
    /// `time LEVEL scope: message key=value`.
    #[default]
    Text,
    /// One JSON object per line.
    Json,
}

/// What an audited action came to.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome {
    /// The action happened.
    #[default]
    Done,
    /// The actor was not allowed to.
    Denied,
    /// The action was attempted and failed.
    Failed,
}

/// A borrowed field value, shown in its JSON form; `Show` renders any `Display` at write time.
#[non_exhaustive]
#[derive(Clone, Copy)]
pub enum Value <'a> {
    /// No value; JSON `null`.
    Null,
    /// A flag.
    Bool(bool),
    /// A signed integer.
    Int(i64),
    /// An unsigned integer.
    Uint(u64),
    /// A real; JSON `null` when not finite.
    Float(f64),
    /// Borrowed text.
    Str(&'a str),
    /// Anything that can display itself.
    Show(&'a dyn Display),
}

/// An owned value a logger carries for the life of a handle.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum Owned {
    /// No value; JSON `null`.
    Null,
    /// A flag.
    Bool(bool),
    /// A signed integer.
    Int(i64),
    /// An unsigned integer.
    Uint(u64),
    /// A real; JSON `null` when not finite.
    Float(f64),
    /// Owned text.
    Text(Box<str>),
}

/// A field bound to a logger handle: a static key with an owned value, rendered before the record's own fields.
#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    pub(crate) key:   &'static str,
    pub(crate) value: Owned,
}

/// One key with its value.
#[derive(Clone, Copy)]
pub struct Field <'a> {
    pub(crate) key:   &'a str,
    pub(crate) value: Value<'a>,
}

/// Up to [`Fields::CAPACITY`] fields on the stack; what does not fit is counted in a last `dropped` field.
#[derive(Clone, Copy)]
pub struct Fields <'a> {
    pub(crate) items:   [Field<'a>; CAPACITY + 1],
    pub(crate) count:   usize,
    pub(crate) dropped: u64,
}

/// A moment in UTC as time since the Unix epoch, shown as RFC 3339 with microseconds.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Stamp {
    pub(crate) since: Duration,
}

/// What a record is: an event at a level, or an audited action with its outcome.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nature {
    /// An event at this level.
    Event(Level),
    /// An audited action that came to this.
    Audit(Outcome),
}

/// One record as a sink receives it: everything borrowed, rendered through a [`Shape`].
#[derive(Clone, Copy)]
pub struct Record <'a> {
    pub(crate) stamp:   Stamp,
    pub(crate) scope:   &'a str,
    pub(crate) kind:    Nature,
    pub(crate) actor:   &'a str,
    pub(crate) message: &'a dyn Display,
    pub(crate) bound:   &'a [Binding],
    pub(crate) fields:  &'a [Field<'a>],
}

/// A destination for records; implement it to add one.
pub trait Sink: Send {

    /// Writes one record.
    ///
    /// # Errors
    /// The I/O failure of the destination.
    fn write ( &mut self, record: &Record<'_> ) -> IoResult<()>;

    /// Makes everything written so far durable where that means something.
    ///
    /// # Errors
    /// The I/O failure of the destination.
    fn flush ( &mut self ) -> IoResult<()>;

}

/// The standard error stream, one write per record.
#[derive(Debug, Clone, Copy)]
pub struct Stderr {
    pub(crate) shape: Shape,
}

/// The standard output stream, one write per record.
#[derive(Debug, Clone, Copy)]
pub struct Stdout {
    pub(crate) shape: Shape,
}

/// An append-only file, one write per record, so lines stay whole even across processes;
/// optionally rotated by size into `path.1`, `path.2`, up to the kept count.
#[derive(Debug)]
pub struct Logfile {
    pub(crate) shape:   Shape,
    pub(crate) handle:  Handle,
    pub(crate) path:    PathBuf,
    pub(crate) written: u64,
    pub(crate) limit:   u64,
    pub(crate) keep:    usize,
}

/// An in-memory sink shared through its clones: what tests and probes read back.
#[derive(Debug, Clone)]
pub struct Capture {
    pub(crate) shape: Shape,
    pub(crate) store: Arc<Mutex<Vec<u8>>>,
}

/// JSON text encoding, the subset a log line needs.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Json;

pub(crate) struct Counted <'h> {
    pub(crate) handle:  &'h mut Handle,
    pub(crate) written: u64,
}

pub(crate) struct Line <'w, W: Write> {
    pub(crate) writer: &'w mut W,
    pub(crate) buffer: [u8; LINE],
    pub(crate) len:    usize,
    pub(crate) error:  Option<IoError>,
}

pub(crate) struct Inner {
    pub(crate) level:   AtomicU8,
    pub(crate) dropped: AtomicU64,
    pub(crate) sinks:   Sinks,
    pub(crate) audits:  Sinks,
}

/// A logger: cheap to clone, safe to share, named by scope; silent until [`Setup`] gave it sinks.
#[derive(Clone)]
pub struct Log {
    pub(crate) inner: Arc<Inner>,
    pub(crate) scope: &'static str,
    pub(crate) bound: Arc<[Binding]>,
}

/// Configures a [`Log`]: the threshold level, the event sinks and the audit sinks.
pub struct Setup {
    pub(crate) level:  Level,
    pub(crate) sinks:  Vec<Box<dyn Sink>>,
    pub(crate) audits: Vec<Box<dyn Sink>>,
}

/// An event under construction; `emit` counts a failed write, `commit` returns it.
#[derive(Clone, Copy)]
pub struct Event <'a, M: Display> {
    pub(crate) log:     &'a Log,
    pub(crate) level:   Level,
    pub(crate) message: M,
    pub(crate) fields:  Fields<'a>,
}

/// An audited action under construction; it ignores the level, and `commit` is the call to make.
#[derive(Clone, Copy)]
pub struct Audit <'a> {
    pub(crate) log:     &'a Log,
    pub(crate) action:  &'a str,
    pub(crate) actor:   &'a str,
    pub(crate) outcome: Outcome,
    pub(crate) fields:  Fields<'a>,
}
