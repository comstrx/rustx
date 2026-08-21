//! Purpose-built types and type utilities for this workspace.

/// Entry point for purpose-built typing utilities.
pub struct Typing;

impl Typing {
    /// Returns the canonical greeting.
    #[must_use]
    pub const fn hello_world() -> &'static str {
        "Hello, world!"
    }
}
