# Proof, Benchmarks, and World-Class Claims

**Load when:** making performance/safety claims, designing benchmarks, or
setting regression policy.

**Output:** a claim-to-evidence map and a reproducible dossier with honest
losses and uncertainty.

## The evidence hierarchy

```text
Level 0 — hypothesis      plausible design, no performance claim
Level 1 — local proof     correct on target cases; microbenchmark win on one machine
Level 2 — class proof     representative datasets, adversarial cases, multiple
                          competitors, allocation/counter analysis, major targets
Level 3 — world-class     reproducible suite, exact versions/configs, broad
                          workload matrix, independent raw data, guarded regressions
```

**Never promote evidence to a higher level in prose.** "Fastest," "zero-cost,"
"lock-free," "constant-time," "allocation-free," and "safe" are testable
contracts, not adjectives.

## The proof portfolio

| Claim                   | Minimum evidence                                |
| ----------------------- | ----------------------------------------------- |
| semantic equivalence    | oracle/differential/property tests              |
| memory soundness        | safety case + Miri/sanitizers as applicable     |
| concurrency correctness | memory-order proof + model/stress assets        |
| parser robustness       | adversarial corpus + fuzz + resource limits     |
| fastest/top-tier        | reproducible competitor benchmark matrix        |
| zero allocation         | allocator instrumentation on the declared path  |
| no copy                 | byte-journey audit + instrumentation/inspection |
| stable API              | semver checks + canonical path tests            |

Safety proof and speed proof are independent tracks that converge before a
world-class claim.

## The benchmark contract

Every serious benchmark records: operation and semantics; input corpus and
generation seed; size/distribution classes; trusted/validated mode; warm/cold
state; preparation included or excluded per real reuse; concurrency and
affinity; allocator; toolchain and LLVM version; target triple/CPU;
RUSTFLAGS/profile/LTO/features; competitor versions and features; measurement
tool and statistics; source commit. Without this metadata a number is a clue,
not proof.

## Apples-to-apples semantics

Equal semantics or no comparison: UTF-8 validation aligned; duplicate-key
policy aligned; checked vs unchecked aligned; borrowed vs owned output
aligned; parse-to-typed vs parse-to-DOM distinguished; hash security mode
aligned; preparation cost included per real reuse. Never beat a safe
competitor with an unsafe API and call it the same class. Never benchmark
`target-cpu=native` against generically built competitors and market it as
the portable default.

## The workload matrix

Sizes: `0 | 1 | tiny | cache-line | 1 KiB | 8 KiB | 64 KiB | 1 MiB | streaming`.
Shapes: dense/no matches, ASCII/mixed Unicode, predictable/unpredictable
branches, uniform/skewed keys, read-only/balanced/write-heavy, valid/malformed
early/late, shallow/deep nesting, prepared/one-shot. Adversarial:
collision-heavy keys, repeated prefixes, filter-defeating needles,
escape-heavy strings, worst-case numeric tokens, stalled readers, contention
hot spots, capacity boundaries.

## Metrics and hygiene

Measure beyond wall time: throughput; p50/p95/p99 latency; allocations and
bytes; peak/transient memory; bytes copied; cycles/instructions/IPC;
branches and mispredictions; cache/TLB misses; syscalls; code size; compile
time when API strategy affects it; scalability curves for concurrent engines.

Hygiene: consume outputs with `black_box`; separate setup from operation per
the workload contract; rotate inputs when cache warmth would falsify the use
case; pin frequency/governor/affinity and report it; report distributions,
not best times; check generated code after surprising results; re-run on
clean builds and a second machine for major claims; keep raw artifacts;
verify outputs (dead-code elimination cheats silently).

## Competitor selection

A "fastest in the world" claim requires the actual leaders of the exact
class: discover current leaders, read their methodology, use their best
supported configuration, include specialized competitors, record semantic
differences, re-evaluate periodically — leadership changes.

## Regression policy

Thresholds exceed benchmark noise and reflect product importance. Store a
baseline **distribution**, not a single number. Never fail on nanosecond
noise; never ignore a consistent 3–5% hot-path loss because tests pass.
Track common-case, adversarial, memory, code size, compile time. Formal
regression gates run on demand; benchmark assets stay maintained. During
exploration, record suspected regressions without derailing design.

## Formal verification escalation

For safety-critical kernels: Kani (bounded, bit-precise harnesses), a
Verus/Creusot-class tool (specification-driven proofs), Miri/cargo-careful/
sanitizers (dynamic UB classes), Loom/Shuttle (interleavings). Use only with
an explicit property, bounded model, trusted-computing-base statement, and a
counterexample plan. A verifier proves the encoded model — not unmodeled FFI,
compiler, or hardware. Verification tooling may use a lab toolchain while the
shipped crate remains stable 1.98.

## The benchmark dossier

Each flagship engine eventually ships `BENCHMARKS.md`: semantic contract;
competitors and versions; hardware/toolchains; corpora and adversarial cases;
commands/configuration; raw-result locations; interpretation; **known losses
and trade-offs**; claim level. Publishing where RustX loses — and why — is
what makes the wins credible.
