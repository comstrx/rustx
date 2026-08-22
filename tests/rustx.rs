//! Public-API contract tests for the `rustx` facade.
//!
//! The facade's contract is its paths: this target asserts that every crate is
//! reachable through `rustx::` and through `rustx::prelude`, which is the only
//! thing the facade promises.

#[cfg(test)]
mod tests {
    use rustx::{base::Typing as ReExported, prelude::*};

    #[test]
    fn base_is_reachable_through_the_facade() {
        assert_eq!(ReExported::hello_world(), "Hello, world!");
    }

    #[test]
    fn facade_prelude_exposes_the_base_vocabulary() {
        assert_eq!(Typing::hello_world(), ReExported::hello_world());
    }

    #[test]
    fn facade_re_export_is_the_same_type() {
        let value: Typing = ReExported::default();

        assert_eq!(value, Typing::default());
    }
}
