use typing::Typing;

#[allow(
    clippy::disallowed_macros,
    clippy::print_stdout,
    reason = "This executable example intentionally demonstrates the returned greeting."
)]
fn main() {
    println!("{}", Typing::hello_world());
}
