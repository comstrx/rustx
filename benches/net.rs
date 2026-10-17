//! What the transport costs off the wire: parsing an address, printing one, a stop check, a listener bind.

use criterion::{Criterion, criterion_group, criterion_main};
use engine::net::{Address, Bind, Listener};
use engine::thread::Stop;
use core::hint::black_box;

fn bench_net ( harness: &mut Criterion ) {

    let mut group = harness.benchmark_group("net");
    let stop      = Stop::new();
    let address   = Address::parse("127.0.0.1:0").ok();

    group.bench_function("address/parse-tcp", |bencher| bencher.iter(|| Address::parse(black_box("10.1.2.3:8080"))));
    group.bench_function("address/parse-name", |bencher| bencher.iter(|| Address::parse(black_box("api.internal:9000"))));
    group.bench_function("address/display", |bencher| bencher.iter(|| address.as_ref().map(ToString::to_string)));
    group.bench_function("stop/raised", |bencher| bencher.iter(|| black_box(&stop).raised()));
    group.bench_function("listener/bind-drop", |bencher| bencher.iter(|| address.as_ref().map(|address| Listener::bind(address, Bind::new()).map(|listener| listener.local().port()))));
    group.finish();

}

criterion_group!(benches, bench_net);
criterion_main!(benches);
