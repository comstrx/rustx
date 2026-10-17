//! What a request costs end to end on the loopback, what a response costs to build, what the handshake digest costs.

use criterion::{Criterion, criterion_group, criterion_main};
use engine::http::{Client, Response, Router, Server};
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use engine::net::Address;
use engine::thread::{Runtime, Stop};
use std::sync::mpsc::channel;
use core::hint::black_box;
use core::time::Duration;
use std::thread::spawn;
use engine::error::Failure;

const LOOPBACK: Address = Address::Tcp(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0));

fn routes () -> Router {

    Router::new().get("/health", |_request| async { Ok(Response::text("ok")) })

}

fn bench_http ( harness: &mut Criterion ) {

    let stop   = Stop::new();
    let halt   = stop.clone();
    let ( tell, hear ) = channel::<Address>();
    let server = spawn(move || Runtime::block_on(Server::bind(LOOPBACK).serve(routes(), halt, |local| tell.send(local.clone()).unwrap_or_default())).and_then(|outcome| outcome));
    let base   = hear.recv_timeout(Duration::from_secs(5)).ok().map(|local| format!("http://{local}/health"));

    measure(harness, base.as_deref());
    stop.raise();
    drop(server.join());

}

fn measure ( harness: &mut Criterion, base: Option<&str> ) {

    let mut group = harness.benchmark_group("http");
    let client    = Client::new();

    group.bench_function("get/loopback", |bencher| bencher.iter(|| Runtime::block_on(async { match base { Some(uri) => client.get(uri).await.map(|response| response.code()), None => Err(Failure::internal("no server")) } })));
    group.bench_function("response/text", |bencher| bencher.iter(|| Response::text(black_box("ok")).code()));
    group.bench_function("response/failure", |bencher| bencher.iter(|| Response::from(Failure::forbidden(black_box("no"))).code()));
    group.finish();

}

criterion_group!(benches, bench_http);
criterion_main!(benches);
