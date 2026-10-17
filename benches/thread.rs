//! What concurrency costs: a fresh runtime against a kept executor, a task spawned and joined, a deadline that holds.

use criterion::{Criterion, criterion_group, criterion_main};
use engine::thread::{Executor, Runtime, Task, Time};
use core::hint::black_box;
use core::time::Duration;

async fn joined () -> Option<u8> {

    Task::spawn(async { black_box(3) }).join().await.ok()

}

fn bench_thread ( harness: &mut Criterion ) {

    let mut group = harness.benchmark_group("thread");
    let executor  = Executor::new().ok();

    group.bench_function("runtime/block_on", |bencher| bencher.iter(|| Runtime::block_on(async { black_box(1) }).ok()));
    group.bench_function("executor/block_on", |bencher| bencher.iter(|| executor.as_ref().map(|kept| kept.block_on(async { black_box(1) }))));
    group.bench_function("task/spawn-join", |bencher| bencher.iter(|| executor.as_ref().map(|kept| kept.block_on(joined()))));
    group.bench_function("time/within", |bencher| bencher.iter(|| executor.as_ref().map(|kept| kept.block_on(Time::within(Duration::from_secs(1), async { black_box(2) })))));
    group.finish();

}

criterion_group!(benches, bench_thread);
criterion_main!(benches);
