//! What a form costs: all rules passing, and all rules failing into one failure.

use criterion::{Criterion, criterion_group, criterion_main};
use core::hint::black_box;
use engine::validate::Form;

fn bench_validate ( harness: &mut Criterion ) {

    let mut group = harness.benchmark_group("validate");

    group.bench_function("clean", |bencher| bencher.iter(|| {

        let mut form = Form::new();

        form.text("email", black_box("someone@example.com")).required().email().max(255);
        form.text("name", black_box("Someone")).required().min(2).max(64);
        form.number("age", black_box(30)).between(18, 120);

        form.finish().is_ok()

    }));

    group.bench_function("failing", |bencher| bencher.iter(|| {

        let mut form = Form::new();

        form.text("email", black_box("nope")).required().email();
        form.text("name", black_box("")).required();
        form.number("age", black_box(7)).at_least(18);

        form.finish().err().map(|failure| failure.problems().len())

    }));

    group.finish();

}

criterion_group!(benches, bench_validate);
criterion_main!(benches);
