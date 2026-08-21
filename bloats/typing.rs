#[cfg(feature = "bloat-typing")]
fn main() {
    std::hint::black_box(typing::Typing::hello_world());
}

#[cfg(not(feature = "bloat-typing"))]
#[allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::exit,
    clippy::print_stderr,
    reason = "The adapter must fail loudly when its required feature is disabled."
)]
fn main() {
    eprintln!("enable: --features bloat-typing");
    std::process::exit(2);
}
