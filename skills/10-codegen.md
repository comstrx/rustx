# Compiler, Codegen, and Binary Optimization

**Load when:** tuning LLVM output, inlining, target features, PGO/BOLT,
monomorphization, or release profiles.

**Output:** a measured code-generation hypothesis with portable and
controlled-target policy separated.

## Feed LLVM facts it can use

The compiler optimizes facts, not intentions: non-aliasing mutable borrows,
exact slice lengths, const generic sizes, concrete monomorphized types, cold
error functions, target features isolated per kernel, loops with independent
accumulators, no hidden calls in inner loops, explicit contiguous storage.
Never assume an optimization happened because the source "looks optimizable."

## The optimization evidence ladder

Use the cheapest evidence that answers the question — never start at assembly
when the representation is wrong:

```text
source-level cost ledger
→ benchmark counters / allocation counts
→ profiler flame/call graph
→ hardware counters (cache, branch, cycles, bandwidth)
→ generated assembly / LLVM IR / MIR
→ PGO profile and post-link layout
```

Inspect generated artifacts when a claim depends on vectorization,
bounds-check elimination, inlining, allocation removal, branchless lowering,
enum layout, or atomic selection.

## Bounds-check elimination

Prefer transformations that prove bounds naturally: iterate slices directly;
zip equal-length slices after one length check; `chunks_exact`/array chunks;
hoist one range check before the loop; split APIs that establish disjoint
ranges; keep index/length relationships simple. `get_unchecked` only when
codegen shows checks remain material and the invariant is easy to prove — an
unchecked access that duplicates optimizer work is only extra unsafety.
Iterators are not a religion: in measured hot loops, indexing, chunking, or
SIMD kernels win when proven; keep the clearest form when codegen is equal.

## Inlining policy

`#[inline]` for small cross-crate wrappers, generic combinators, dispatch
facades, accessors (the workspace lint requires it on public items — design
accordingly). `#[inline(always)]` rarely, for tiny measured kernels where
non-inlining is consistently harmful. `#[inline(never)]` + `#[cold]` for cold
paths and code-size control — error handling is a candidate, not an automatic
target; a wrong cold hint hurts. Inlining is a hypothesis: inspect size and
performance.

## Monomorphization budget

Generics produce perfect hot code and terrible binary topology. Keep large
generic functions thin, delegating to non-generic kernels; normalize inputs
early; enums/function pointers for cold variability; no generic parameters
that do not affect hot code; measure `.text` size and compile time; watch
duplicate instantiations.

## Workspace profiles (locked in root `Cargo.toml`)

```text
release          opt3 · fat LTO · CGU 1 · panic=abort · strip · no overflow checks
release-checked  release + debug-assertions + overflow-checks (correctness proof builds)
profiling        release + full debug info, no strip (perf tools)
dev              deps at opt3 (fast iteration with realistic dep speed)
```

Profiles are workspace/artifact concerns — library semantics never assume
downstream profile settings, and libraries stay sound under unwinding even
though local release aborts. Benchmark/bench profile inherits release.
`trim-paths` is still unstable on cargo 1.98 — do not add it.

## Target portfolio

Separate artifacts, never confused with each other:

- **Portable library**: baseline target floor + runtime-dispatched kernels
  (see [05-simd.md](05-simd.md)). Never global `-C target-feature`.
- **Controlled service binary**: fleet CPU tuning (`target-cpu`), full LTO,
  allocator selected by measurement, PGO — the artifact must not run on weaker
  CPUs and the deployment metadata records the CPU contract.
- **Benchmark artifact**: built for the measured machine; never publish
  numbers from a configuration users cannot obtain without stating it.

## PGO and BOLT

PGO protocol: build instrumented with the same flags → run representative
production-shaped workloads (not one happy-path microbenchmark) → merge →
rebuild with profile use → validate coverage/staleness → benchmark
independently. A biased profile optimizes the wrong world. BOLT reorders
final machine code post-link on supported ELF platforms — a deployment
optimization: representative profiles, preserved symbols, measured startup/
throughput/tail, validated unwind behavior. PGO and BOLT are complementary;
measure the combination. Both come after representation and algorithm are
right.

## Rust 1.98 codegen notes

- Reserved runtime symbols: functions named like `memcpy`, `memset`, `strlen`,
  or allocator hooks must never be exported accidentally; keep kernel names
  namespaced and audit `#[unsafe(no_mangle)]`/`export_name`.
- `repr(transparent)` is validated more strictly: one-field genuine ABI
  wrappers only — never a "same layout probably" annotation.
- Stable `core::hint::cold_path()` and `select_unpredictable` are available;
  stable `likely`/`unlikely` functions are not — build a local helper from
  `cold_path` only if benchmarks prove value.
