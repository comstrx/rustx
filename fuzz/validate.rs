//! Any text through every text rule, any number through every number rule, and one failure out.

#![no_main]
#![expect(clippy::disallowed_methods, reason = "the libFuzzer entry macro expands to `expect`")]

use libfuzzer_sys::fuzz_target;
use engine::validate::Form;

fuzz_target!(|data: &[u8]| {

    let text   = String::from_utf8_lossy(data);
    let number = data.first_chunk::<8>().map_or(0, |chunk| i64::from_le_bytes(*chunk));

    let mut form = Form::new();

    form.text("a", &text).required().min(1).max(64).ascii();
    form.text("b", &text).email();
    form.text("c", &text).slug();
    form.text("d", &text).one_of(&["x", "y"]);
    form.number("e", number).between(-10, 10).positive();
    form.number("f", number).at_least(0).at_most(0);

    let problems = form.problems().len();

    assert!(problems <= 6);
    assert_eq!(form.is_clean(), problems == 0);

    match form.finish() {
        Ok(())       => assert_eq!(problems, 0),
        Err(failure) => {
            assert_eq!(failure.problems().len(), problems);
            assert_eq!(failure.key(), "invalid_fields");
            assert!(failure.wire().to_string().contains("\"fields\":{"));
        }
    }

});
