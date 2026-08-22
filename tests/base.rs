//! Public-API contract tests for `rustx-base`.
//!
//! This target links the crate the way a downstream consumer does, so anything
//! asserted here is part of the published surface.

#[cfg(test)]
mod tests {
    use rustx_base::prelude::*;

    #[test]
    fn hello_world_returns_the_canonical_greeting() {
        assert_eq!(Typing::hello_world(), "Hello, world!");
    }

    #[test]
    fn hello_world_matches_the_public_constant() {
        assert_eq!(Typing::hello_world(), Typing::GREETING);
    }

    #[test]
    fn hello_world_is_stable_across_calls() {
        assert_eq!(Typing::hello_world().as_ptr(), Typing::hello_world().as_ptr());
    }

    #[test]
    fn namespace_type_is_zero_sized() {
        assert_eq!(size_of::<Typing>(), 0);
    }

    #[test]
    fn prelude_exposes_the_namespace_type() {
        let value = Typing::default();

        assert_eq!(value, Typing::default());
    }
}
