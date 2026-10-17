/// What the engine promises a client: every code with its HTTP status and exit code, and every
/// failure the engine declares, each key under one code.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Contract;
