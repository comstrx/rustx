#[cfg(feature = "bloat-rustx")]
fn main() {
    std::hint::black_box(rustx::typing::Typing::hello_world());
}

#[cfg(not(feature = "bloat-rustx"))]
#[allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::exit,
    clippy::print_stderr,
    reason = "The adapter must fail loudly when its required feature is disabled."
)]
fn main() {
    eprintln!("enable: --features bloat-rustx");
    std::process::exit(2);
}
