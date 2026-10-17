//! A confined path never leaves its root and never panics, whatever the input.

#![no_main]
#![expect(clippy::disallowed_methods, reason = "the libFuzzer entry macro expands to `expect`")]

use std::path::{Component, Path as StdPath};
use libfuzzer_sys::fuzz_target;
use core::hint::black_box;
use engine::file::Path;

fuzz_target!(|data: &[u8]| {

    let text   = String::from_utf8_lossy(data);
    let root   = StdPath::new("/root/dir");
    let inside = StdPath::new(&*text);

    if let Ok(confined) = Path::confine(root, inside) {

        assert!(confined.starts_with(root));
        assert!(!confined.components().any(|component| matches!(component, Component::ParentDir | Component::CurDir)));

    }

    black_box(Path::normalize(inside));
    black_box(Path::extension(inside));
    black_box(Path::stem(inside));
    black_box(Path::is_hidden(inside));

});
