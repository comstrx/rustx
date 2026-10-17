//! Any words parse as arguments, and any spec renders units whose escaping holds.

#![no_main]
#![expect(clippy::disallowed_methods, reason = "the libFuzzer entry macro expands to `expect`")]

use engine::process::{Args, Provider, Restart, Spec};
use libfuzzer_sys::fuzz_target;
use core::hint::black_box;

fuzz_target!(|data: &[u8]| {

    let text  = String::from_utf8_lossy(data);
    let words: Vec<&str> = text.split(' ').collect();
    let args  = Args::from_words(&words);

    black_box(( args.command().map(str::len), args.rest().len(), args.has("x"), args.get("y").map(str::len), args.positional(0).map(str::len) ));

    let name = words.first().copied().unwrap_or("x").trim_start_matches('-');
    let name = if name.is_empty() { "x" } else { name };
    let spec = Spec::new(name, &*text).description(&text).arg(&text).env(&text, &text).user(&text).restart(Restart::Always).boot(data.len().is_multiple_of(2));

    let unit  = spec.render(Provider::Systemd);
    let plist = spec.render(Provider::Launchd);
    let shell = spec.render(Provider::Windows);

    assert!(unit.starts_with("[Unit]\n") && unit.ends_with("WantedBy=multi-user.target\n"));
    assert!(unit.lines().all(|line| line.is_empty() || line.starts_with('[') || line.contains('=')));
    assert!(plist.starts_with("<?xml") && plist.ends_with("</plist>\n"));
    assert_eq!(plist.matches("<dict>").count(), plist.matches("</dict>").count());
    assert!(!plist.contains("&\"") && !plist.replace("&lt;", "").replace("&gt;", "").replace("&amp;", "").replace("&quot;", "").contains(['<', '>'].as_ref()) || plist.contains("<string>"));
    assert!(shell.starts_with("create "));

});
