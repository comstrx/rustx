/// Entry point for the workspace's type utilities.
///
/// `Typing` is a zero-sized namespace rather than a value type: it carries no
/// state and exists so that every type utility is reached through one stable,
/// discoverable path instead of a loose set of free functions.
///
/// # Examples
///
/// ```
/// use rustx_base::Typing;
///
/// assert_eq!(size_of::<Typing>(), 0);
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct Typing;

impl Typing {
    /// The canonical greeting returned by [`Typing::hello_world`].
    ///
    /// Exposed as a constant so callers can use it in `const` position and in
    /// pattern matches without paying for a function call.
    pub const GREETING: &str = "Hello, world!";

    /// Returns the canonical greeting.
    ///
    /// The result is a `'static` string slice baked into the binary: the call
    /// performs no allocation, no copying, and no work at run time.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustx_base::Typing;
    ///
    /// assert_eq!(Typing::hello_world(), "Hello, world!");
    /// ```
    ///
    /// It is usable wherever a constant is required:
    ///
    /// ```
    /// use rustx_base::Typing;
    ///
    /// const GREETING: &str = Typing::hello_world();
    ///
    /// assert_eq!(GREETING, Typing::GREETING);
    /// ```
    #[inline]
    #[must_use]
    pub const fn hello_world() -> &'static str {
        Self::GREETING
    }
}

#[cfg(test)]
mod tests {
    use super::Typing;

    #[test]
    fn hello_world_matches_the_greeting_constant() {
        assert_eq!(Typing::hello_world(), Typing::GREETING);
    }

    #[test]
    fn greeting_is_the_canonical_text() {
        assert_eq!(Typing::GREETING, "Hello, world!");
    }

    #[test]
    fn namespace_type_is_zero_sized() {
        assert_eq!(size_of::<Typing>(), 0);
    }

    #[test]
    fn hello_world_is_usable_in_const_position() {
        const GREETING: &str = Typing::hello_world();

        assert_eq!(GREETING, Typing::GREETING);
    }
}
