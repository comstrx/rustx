use core::result::Result as StdResult;
use core::error::Error as StdError;
use alloc::borrow::Cow;

/// Why something failed, as a stable vocabulary; each code knows its HTTP status, exit code and level.
#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Code {
    /// The caller misused the interface: a bad flag, a malformed request.
    Usage,
    /// The input is well-formed but wrong.
    Invalid,
    /// Nobody is identified.
    Unauthenticated,
    /// Somebody is identified and not allowed.
    Forbidden,
    /// There is no such thing.
    NotFound,
    /// The thing exists but not for this method.
    Method,
    /// The thing is in another state than the request assumes.
    Conflict,
    /// The thing existed and is gone for good.
    Gone,
    /// A precondition the caller stated does not hold.
    Precondition,
    /// The input is larger than this side accepts.
    TooLarge,
    /// The input is well-formed and understood, and still cannot be acted on.
    Unprocessable,
    /// Too many requests for now.
    Throttled,
    /// The caller gave up first.
    Cancelled,
    /// A bug or an unexpected condition on this side.
    Internal,
    /// The configuration does not let this run.
    Config,
    /// Not implemented here, by design or not yet.
    Unsupported,
    /// A dependency is not answering.
    Unavailable,
    /// A dependency did not answer in time.
    Timeout,
    /// A resource ran out: memory, a buffer, a quota.
    Exhausted,
}

/// One failure a crate declares once: the key a client translates by, the code it always carries,
/// and the detail it says unless a site says more.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fault {
    pub(crate) key:    &'static str,
    pub(crate) code:   Code,
    pub(crate) detail: &'static str,
}

/// The failures this crate raises on its own: conversions from the standard library's errors.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Faults;

/// Owned or static text for a detail or a frame; static text costs nothing.
pub type Detail = Cow<'static, str>;

/// What this crate's functions return.
pub type Result <T, E = Failure> = StdResult<T, E>;

pub(crate) type Source = Box<dyn StdError + Send + Sync>;

/// The one failure: a code, a key, a detail, and what it wraps.
pub struct Failure {
    pub(crate) code:   Code,
    pub(crate) key:    Option<&'static str>,
    pub(crate) detail: Detail,
    pub(crate) extra:  Option<Box<Extra>>,
}

/// One field that failed validation: which, why, and in words.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Problem {
    pub(crate) field:  Detail,
    pub(crate) key:    &'static str,
    pub(crate) detail: Detail,
}

pub(crate) struct Extra {
    pub(crate) frames:   Vec<Detail>,
    pub(crate) source:   Option<Source>,
    pub(crate) field:    Option<Detail>,
    pub(crate) retry:    Option<u32>,
    pub(crate) problems: Vec<Problem>,
}

/// The whole chain for people: code, detail, every frame and every cause, one per line.
#[derive(Debug, Clone, Copy)]
pub struct Report <'a> {
    pub(crate) failure: &'a Failure,
}

/// The failure as a JSON body for the wire: `{"code":…,"key":…,"message":…}`.
#[derive(Debug, Clone, Copy)]
pub struct Wire <'a> {
    pub(crate) failure: &'a Failure,
}

/// What `main` returns: the failure's exit code, reported on standard error or kept quiet.
#[derive(Debug)]
pub struct Exit {
    pub(crate) failure: Option<Failure>,
    pub(crate) report:  bool,
}

/// Turns a `Result` of any error, or an `Option`, into a [`Result`] of [`Failure`].
pub trait Fail <T> {

    /// A failure with this code wrapping the error, or for the missing value.
    ///
    /// # Errors
    /// The failure built from `self`.
    fn or_fail ( self, code: Code ) -> Result<T>;

    /// Adds a frame to the failure, or makes a `NotFound` failure with this detail for the missing value.
    ///
    /// # Errors
    /// The failure built from `self`.
    fn context ( self, frame: impl Into<Detail> ) -> Result<T>;

    /// The failure `make` builds, wrapping the error or standing in for the missing value.
    ///
    /// # Errors
    /// The failure built from `self`.
    fn or_fail_with ( self, make: impl FnOnce() -> Failure ) -> Result<T>;

}
