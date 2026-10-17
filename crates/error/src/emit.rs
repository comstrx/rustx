use std::io::Result as IoResult;
use log::{Log, Outcome, Value};

use crate::Failure;

impl Failure {

    /// Logs the failure at its code's level with its code, key, HTTP status, exit status and cause;
    /// a failed write is counted by the logger.
    #[inline]
    pub fn emit ( &self, log: &Log ) {

        if self.log(log).is_err() { log.warn("a failure could not be logged").emit(); }

    }

    /// Logs the failure like [`Failure::emit`] and returns the sink's failure instead of counting it.
    ///
    /// # Errors
    /// The I/O failure of a sink.
    #[inline]
    pub fn log ( &self, log: &Log ) -> IoResult<()> {

        log.event(self.level(), self)
            .field("code", self.code.key())
            .field("key", self.key())
            .field("http", self.http())
            .field("exit", self.exit())
            .field("retryable", self.retryable())
            .field("cause", self.origin().map(|cause| Value::show(cause)))
            .commit()

    }

    /// Journals the failed action by this actor with the failure's code, key and text.
    ///
    /// # Errors
    /// The I/O failure of an audit sink.
    #[inline]
    pub fn audit ( &self, log: &Log, action: &str, actor: &str ) -> IoResult<()> {

        log.audit(action)
            .actor(actor)
            .outcome(Outcome::Failed)
            .field("code", self.code.key())
            .field("key", self.key())
            .show("detail", self)
            .commit()

    }

}
