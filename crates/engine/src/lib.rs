//! The one door into the foundation: each capability as a module whose consumer surface is named
//! item by item, a [`prelude`] of their vocabulary, and the [`Contract`] of every code and every
//! declared failure a client translates by. The heavy capabilities are features a consumer names:
//! `file`, `net`, `http` (with `net`) and `process`; `alloc` adds the global allocator.

#![forbid(unsafe_code)]

mod arch;
mod contract;

pub use crate::arch::Contract;

/// Structured events and audit records as values, written only through a sink.
pub mod log {

    pub use log::{Audit, Binding, Capture, Event, Field, Fields, Json, Level, Log, Logfile, Nature, Outcome, Owned, Record, Setup, Shape, Sink, Stamp, Stderr, Stdout, Value};

}

/// The one failure every capability returns, its codes, and the faults declared once.
pub mod error {

    pub use error::{Code, Detail, Exit, Fail, Failure, Fault, Faults, Problem, Report, Result, Wire};

}

/// Bytes without surprises: shared bytes, a stack, a buffer, a pool, an arena, a cursor, SIMD lanes.
pub mod memory {

    pub use memory::{Arena, Buffer, Bytes, Cursor, Faults, Lease, Pool, Simd, Space, Split, Stack, Store};

    #[cfg(feature = "alloc")]
    pub use memory::Alloc;

}

/// Text into values: numbers, flags, sizes, durations, percent-encoding, queries, lines and lists.
pub mod parse {

    pub use parse::{Faults, Lines, List, Pair, Parse, Percent, Query};

}

/// Forms that collect every failed field into one failure, each field under a declared fault.
pub mod validate {

    pub use validate::{Check, Faults, Form, Number, Text};

}

/// Concurrency: a runtime per worker, a kept executor, tasks, deadlines and the stop signal.
pub mod thread {

    pub use thread::{Executor, Faults, Runtime, Stop, Task, Time, Workers};

}

/// Randomness from the operating system's secure source, and UUIDs of version 4 and 7.
pub mod rand {

    pub use rand::{Faults, Random, Uuid};

}

/// Digests and encodings, unguessable tokens, and secrets compared in constant time.
pub mod crypto {

    pub use crypto::{Digest, Secret, Token};

}

/// Files, directories and paths confined to a root, every call asynchronous.
#[cfg(feature = "file")]
pub mod file {

    pub use file::{Dir, Entry, Faults, File, Info, Kind, Path, Reader, Writer};

}

/// Addresses, listeners with accept-time hooks, streams, dialing with a deadline and resolution.
#[cfg(feature = "net")]
pub mod net {

    pub use net::{Address, Bind, Dial, Faults, Hook, Listener, Peer, Protocol, Resolve, Stream};

}

/// HTTP on one thread per core: routes, middleware, requests and responses, a client, `WebSocket`s.
#[cfg(feature = "http")]
pub mod http {

    pub use http::{Body, Client, Faults, Header, Ident, Limit, Message, Method, Middleware, Next, Param, Params, Pending, Posted, Request, Response, Router, Server, Settings, Socket, Timeout, Trace, Upgrade, Version};

}

/// What a command-line tool stands on: boot, arguments, environment, programs, the machine, packages, services.
#[cfg(feature = "process")]
pub mod process {

    pub use process::{Args, Boot, Cargo, Env, Faults, Flag, Manager, Named, Output, Package, Pid, Platform, Provider, Restart, Run, Service, Spec, State, System, Tool, Var};

}

/// Every capability's vocabulary in one.
pub mod prelude {

    pub use validate::prelude::*;
    pub use memory::prelude::*;
    pub use thread::prelude::*;
    pub use crypto::prelude::*;
    pub use error::prelude::*;
    pub use parse::prelude::*;
    pub use rand::prelude::*;
    pub use log::prelude::*;

    #[cfg(feature = "process")]
    pub use process::prelude::*;

    #[cfg(feature = "file")]
    pub use file::prelude::*;

    #[cfg(feature = "http")]
    pub use http::prelude::*;

    #[cfg(feature = "net")]
    pub use net::prelude::*;

}
