//! What a file costs: a small whole read, a read into an engine buffer, an atomic write, path confinement.

use criterion::{Criterion, criterion_group, criterion_main};
use std::path::Path as StdPath;
use engine::thread::Executor;
use engine::file::{Dir, File, Path};
use core::hint::black_box;
use std::env::temp_dir;
use std::process::id;
use engine::memory::Stack;

fn bench_file ( harness: &mut Criterion ) {

    let mut group = harness.benchmark_group("file");

    let runtime = Executor::new().ok();
    let dir     = temp_dir().join(format!("engine-bench-file-{}", id()));
    let path    = dir.join("small.txt");

    if let Some(runtime) = &runtime { drop(runtime.block_on(async { drop(Dir::ensure(&dir).await); File::write(&path, &[b'x'; 4096]).await })); }

    group.bench_function("read/4k", |bencher| bencher.iter(|| runtime.as_ref().map(|runtime| runtime.block_on(File::read(black_box(&path))).map(|bytes| bytes.len()))));
    group.bench_function("read_into/4k", |bencher| bencher.iter(|| runtime.as_ref().map(|runtime| { let mut stack = Stack::<4096>::new(); runtime.block_on(File::read_into(black_box(&path), &mut stack)) })));
    group.bench_function("write/4k", |bencher| bencher.iter(|| runtime.as_ref().map(|runtime| runtime.block_on(File::write(black_box(&path), &[b'y'; 4096])))));
    group.bench_function("confine", |bencher| bencher.iter(|| Path::confine(black_box(StdPath::new("/srv/site")), black_box(StdPath::new("assets/img/../css/site.css")))));

    if let Some(runtime) = &runtime { drop(runtime.block_on(Dir::remove(&dir))); }

    group.finish();

}

criterion_group!(benches, bench_file);
criterion_main!(benches);
