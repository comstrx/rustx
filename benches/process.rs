//! What a tool's plumbing costs: parsing a command line, typing a variable, rendering a unit, finding a program.

use criterion::{Criterion, criterion_group, criterion_main};
use engine::process::{Args, Env, Provider, Run, Spec};
use core::hint::black_box;

fn bench_process ( harness: &mut Criterion ) {

    let mut group = harness.benchmark_group("process");
    let env       = Env::system();
    let spec      = Spec::new("iam", "/opt/iam/bin/iam").arg("--serve").cwd("/opt/iam").env("IAM_LOG", "info");

    group.bench_function("args/parse-8", |bencher| bencher.iter(|| Args::from_words(black_box(&["tool", "serve", "--port", "8080", "--json", "--name=x", "one", "two"])).get("port").map(str::len)));
    group.bench_function("env/u16-fallback", |bencher| bencher.iter(|| env.u16(black_box("ENGINE_BENCH_UNSET"), 8080)));
    group.bench_function("spec/systemd", |bencher| bencher.iter(|| black_box(&spec).render(Provider::Systemd).len()));
    group.bench_function("spec/launchd", |bencher| bencher.iter(|| black_box(&spec).render(Provider::Launchd).len()));
    group.bench_function("run/which", |bencher| bencher.iter(|| Run::which(black_box("cargo")).map(|path| path.as_os_str().len())));
    group.finish();

}

criterion_group!(benches, bench_process);
criterion_main!(benches);
