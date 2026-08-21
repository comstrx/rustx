use typing::Typing;

#[test]
fn hello_world_contract() {
    assert_eq!(Typing::hello_world(), "Hello, world!");
}
