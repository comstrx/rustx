//! Lanes against naive scans at three sizes, stores against the heap, pools, cursors and splits.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use engine::memory::{Buffer, Cursor, Pool, Simd, Space as _, Stack};
use core::hint::black_box;

const SIZES: [usize; 3] = [64, 1024, 65_536];

fn hay ( len: usize ) -> Vec<u8> {

    let mut bytes = vec![b'.'; len];

    if let Some(last) = bytes.last_mut() { *last = b'x'; }

    bytes

}

fn bench_lanes ( harness: &mut Criterion ) {

    let mut group = harness.benchmark_group("lanes");

    for size in SIZES {

        let bytes = hay(size);
        let twin  = hay(size);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::new("find", size), &bytes, |bencher, bytes| bencher.iter(|| Simd::find(black_box(bytes), black_box(b'x'))));
        group.bench_with_input(BenchmarkId::new("find-naive", size), &bytes, |bencher, bytes| bencher.iter(|| black_box(bytes).iter().position(|&byte| byte == black_box(b'x'))));
        group.bench_with_input(BenchmarkId::new("count", size), &bytes, |bencher, bytes| bencher.iter(|| Simd::count(black_box(bytes), black_box(b'.'))));
        group.bench_with_input(BenchmarkId::new("count-naive", size), &bytes, |bencher, bytes| bencher.iter(|| black_box(bytes).iter().fold(0usize, |acc, &byte| acc + usize::from(byte == black_box(b'.')))));
        group.bench_with_input(BenchmarkId::new("equal", size), &( &bytes, &twin ), |bencher, ( left, right )| bencher.iter(|| Simd::equal(black_box(left), black_box(right))));
        group.bench_with_input(BenchmarkId::new("equal-std", size), &( &bytes, &twin ), |bencher, ( left, right )| bencher.iter(|| black_box(left) == black_box(right)));

    }

    group.finish();

}

fn bench_stores ( harness: &mut Criterion ) {

    let mut group = harness.benchmark_group("stores");
    let pool      = Pool::new(4, 256);
    let line      = "GET /index.html HTTP/1.1\r\n";

    group.bench_function("stack/push-26", |bencher| bencher.iter(|| { let mut stack = Stack::<64>::new(); black_box(stack.push_str(black_box(line)).map(|stack| stack.len())) }));
    group.bench_function("stack/format", |bencher| bencher.iter(|| { let mut stack = Stack::<64>::new(); black_box(stack.format(format_args!("{} {} {}", black_box("GET"), "/index.html", 200)).map(|stack| stack.len())) }));
    group.bench_function("string/push-26", |bencher| bencher.iter(|| { let mut string = String::with_capacity(64); string.push_str(black_box(line)); black_box(string.len()) }));
    group.bench_function("buffer/new-256", |bencher| bencher.iter(|| Buffer::new(black_box(256))));
    group.bench_function("buffer/push-26", |bencher| bencher.iter(|| { let mut buffer = Buffer::new(64); black_box(buffer.push_str(black_box(line)).map(|buffer| buffer.len())) }));
    group.bench_function("pool/lease-push-drop", |bencher| bencher.iter(|| pool.lease().map(|mut lease| lease.push_str(black_box(line)).map(|lease| lease.len()).ok())));
    group.finish();

}

fn bench_reading ( harness: &mut Criterion ) {

    let mut group = harness.benchmark_group("reading");
    let header    = [7, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, b'a', b'b', b'c', b'd', b'\n', b'r', b'e', b's', b't'];
    let csv       = "alpha,beta,gamma,delta,epsilon,zeta,eta,theta,".repeat(8);

    group.bench_function("cursor/header", |bencher| bencher.iter(|| {

        let mut cursor = Cursor::new(black_box(&header));

        ( cursor.u32_le(), cursor.u64_be(), cursor.take(4), cursor.until(b'\n'), cursor.rest().len() )

    }));

    group.throughput(Throughput::Bytes(csv.len() as u64));
    group.bench_function("split/64-pieces", |bencher| bencher.iter(|| Simd::split(black_box(csv.as_bytes()), b',').count()));
    group.bench_function("split/64-pieces-std", |bencher| bencher.iter(|| black_box(csv.as_bytes()).split(|&byte| byte == b',').count()));
    group.finish();

}

criterion_group!(benches, bench_lanes, bench_stores, bench_reading);
criterion_main!(benches);
