use criterion::{Criterion, criterion_group, criterion_main};
use typing::Typing;

fn bench_typing(c: &mut Criterion) {
    let mut g = c.benchmark_group("typing::bench");

    g.bench_function("typing::hello_world", |b| {
        b.iter(|| {
            std::hint::black_box(Typing::hello_world());
        });
    });

    g.finish();
}

criterion_group!(benches, bench_typing);
criterion_main!(benches);
