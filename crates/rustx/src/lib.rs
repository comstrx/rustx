//! Facade crate for this workspace.

pub use typing;

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_typing() {
        assert_eq!(crate::typing::Typing::hello_world(), "Hello, world!");
    }
}
