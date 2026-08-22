//! Minimal end-to-end use of `rustx-base`.
//!
//! Run with: `cargo run --package examples --example base`

use rustx_base::Typing;

#[expect(
    clippy::disallowed_macros,
    clippy::print_stdout,
    reason = "an executable example demonstrates the value by printing it; libraries still may not"
)]
fn main() {
    println!("{}", Typing::hello_world());
}
