//! Binary-size probe for `rustx-base`.
//!
//! Built by `cargo bloat` to attribute binary size to the crate in isolation.
//! The greeting is mixed with a value the compiler cannot know until run time,
//! so the call survives const-folding and the measurement reflects code that
//! actually ships.

use core::hint::black_box;
use std::env::args_os;

use rustx_base::Typing;

fn main() {
    let argc = args_os().count();

    black_box(Typing::hello_world());
    black_box(argc);
}
