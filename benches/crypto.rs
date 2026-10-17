//! What secrets cost: the `WebSocket` accept key, SHA-1 and Base64 over a kilobyte, a fresh token, a fingerprint, a constant-time comparison.

use criterion::{Criterion, criterion_group, criterion_main};
use engine::crypto::{Digest, Secret, Token};
use core::hint::black_box;

fn bench_crypto ( harness: &mut Criterion ) {

    let mut group = harness.benchmark_group("crypto");
    let kilobyte  = [b'x'; 1024];

    group.bench_function("digest/accept-key", |bencher| bencher.iter(|| Digest::base64(&Digest::sha1(black_box(b"dGhlIHNhbXBsZSBub25jZQ==258EAFA5-E914-47DA-95CA-C5AB0DC85B11")))));
    group.bench_function("sha1/1k", |bencher| bencher.iter(|| Digest::sha1(black_box(&kilobyte))));
    group.bench_function("base64/1k", |bencher| bencher.iter(|| Digest::base64(black_box(&kilobyte))));
    group.bench_function("token/fresh", |bencher| bencher.iter(Token::fresh));
    group.bench_function("token/fingerprint", |bencher| bencher.iter(|| Token::fingerprint(black_box("dGhlIHNhbXBsZSBub25jZQ-AAAA"))));
    group.bench_function("secret/equal-64", |bencher| bencher.iter(|| Secret::equal(black_box(&kilobyte[..64]), black_box(&kilobyte[..64]))));
    group.finish();

}

criterion_group!(benches, bench_crypto);
criterion_main!(benches);
