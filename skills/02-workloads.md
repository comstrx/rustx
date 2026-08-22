# Workloads and Cost Models

**Load when:** selecting an engine, promising performance, or changing a hot
representation.

**Output:** a workload fingerprint, cost ledger, routing strategy, and a
measurable hypothesis.

## The workload contract

No engine is designed before its workload is specified. Write a compact
contract in the design note or module docs:

```text
Operation:          parse JSON object to typed struct
Input sizes:        p50 600 B, p95 8 KiB, max 8 MiB
Reuse:              parser reused per worker
CPU targets:        x86_64-v2 baseline + AVX2, aarch64 NEON
Concurrency:        thread-confined parser; output crosses threads
Latency objective:  p99 under X on target corpus
Memory objective:   <= Y transient bytes per input byte
Threat model:       untrusted network input
Semantics:          strict JSON; duplicate-key policy explicit
```

"Fastest" without a workload contract is semantically empty. The workload
fingerprint that selects an engine:

```text
(size distribution, reuse count, mutation ratio, read/write ratio,
 thread topology, input trust, lifetime phase, ordering requirement,
 latency objective, throughput objective, target CPUs, memory ceiling)
```

Two workloads sharing a method name may require different physical engines:
`Dict` for 8 trusted keys, 8 million hostile keys, and 64 writers are three
products behind one vocabulary.

## The cost ledger

State this ledger before claiming an API is optimized:

| Cost                         | Setup | Per operation | Retained | Evidence |
| ---------------------------- | ----: | ------------: | -------: | -------- |
| allocations                  |       |               |          |          |
| bytes copied/moved           |       |               |          |          |
| scans / passes               |       |               |          |          |
| hashes                       |       |               |          |          |
| branches / mispredict risk   |       |               |          |          |
| pointer-dependent loads      |       |               |          |          |
| atomics / locks              |       |               |          |          |
| syscalls / I/O               |       |               |          |          |
| runtime dispatch             |       |               |          |          |
| UTF/validation               |       |               |          |          |
| future/task bytes            |       |               |          |          |
| code size / monomorphization |       |               |          |          |
| diagnostics / telemetry      |       |               |          |          |

Often-hidden costs that belong in the ledger: future state size, cache/TLB
misses and pointer chasing, allocator synchronization and retained capacity,
instruction-cache pressure from monomorphization, runtime feature detection and
first-use initialization, error metadata policy, cancellation cleanup and retry
amplification, observability cardinality. A design that saves one allocation
but triples future size or code size did not eliminate cost; it moved it.

## The optimization ladder

Apply strictly in order — never jump to step 9 while steps 1–5 remain wasteful:

1. Remove unnecessary work.
2. Change algorithmic complexity.
3. Choose the correct representation.
4. Fuse passes where semantics permit.
5. Reuse memory and precomputation.
6. Improve locality and batching.
7. Expose aliasing/length facts to the compiler.
8. Separate predictable hot and rare cold paths.
9. Add SWAR/SIMD or specialized instructions.
10. Tune codegen, PGO, and binary layout.

## The crossover contract

Every specialization has a crossover point. Model both sides:

```text
T_generic(n) = setup_generic + n × unit_generic
T_special(n) = setup_special + n × unit_special
```

Routing thresholds are data, not folklore: record the benchmark corpus and CPU
class that produced them, and prefer robust plateaus over thresholds that win
by noise at one exact size.

## The small-data law

SIMD setup, dispatch, preprocessing, hashing, and arena acquisition have fixed
costs. Most engines need at least two paths:

```text
small input → compact scalar/unrolled path
large input → vectorized/batched path
```

Thresholds are benchmark-derived per architecture and may differ per operation.

## Shape specialization

Specialize by properties that are cheap to detect and materially alter cost:
empty/singleton, ASCII/mixed UTF-8, tiny/medium/streaming, short/long needle,
low/high cardinality, read-mostly/write-heavy, trusted/validated,
contiguous/segmented, unique/shared ownership, deterministic/randomized.
Dispatch complexity is justified only when the selected paths win enough to
repay it.

## Optimization hypothesis template

Before a non-trivial optimization, state:

```text
Hot operation and workload:
Observed profiler/counter evidence:
Suspected mechanism:
Proposed representation/algorithm/codegen change:
Expected metric movement:
Possible regressions (tiny path, memory, code size, safety, DSL):
Benchmark that can falsify the hypothesis:
Revert condition:
```

Structural performance (zero-copy, arenas, layout, SIMD-ready kernels) is
designed early from first principles. Profiling and benchmarks **choose between
implementations and validate claims** — they are not permission slips to start
caring about performance, and they are not optional for claims.
