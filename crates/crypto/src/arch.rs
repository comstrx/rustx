/// SHA-1 and Base64, computed here without dependencies.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Digest;

/// Opaque tokens for sessions and the like: URL-safe, unguessable, fingerprinted for logs.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token;

/// Secrets compared in time that does not depend on where they differ.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Secret;
