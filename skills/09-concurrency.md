# Concurrency, Async, and Atomics

**Load when:** shared state, concurrent maps, queues, task runtimes, futures,
atomics, or reclamation are involved.

**Output:** a topology-first design with state-machine, cancellation,
ordering, and reclamation proofs.

## Remove sharing before optimizing sharing

The fastest synchronization is none. Decision ladder — move right only when
the workload proves the simpler topology insufficient:

```text
exclusive ownership
→ worker-local partition
→ immutable snapshot / RCU-like read path
→ message passing / batching
→ sharded lock
→ fine-grained synchronization
→ lock-free atomics/reclamation
```

Topology profiles: thread-per-core/worker-local (per-worker arena, cache,
parser, buffers; ownership transfer; routing for cross-worker ops); shared
read-mostly (snapshots, epoch-protected maps; long readers affect
reclamation); shared balanced (sharded locks, short critical sections, batch
APIs); producer/consumer (bounded queues, explicit backpressure, batch
handoff, shutdown protocol). `std::thread::scope` borrows stack data across
threads; `thread_local!` with `Cell`/`RefCell` over `static mut` (which the
lint wall bans anyway).

## Lock doctrine

Lock the smallest coherent state, not every field separately. Keep duration
short and visible. Never run arbitrary user callbacks or blocking I/O under a
lock. Establish lock ordering or avoid nesting. **Lock-across-await is not a
slogan:** a synchronous guard across `.await` is generally incompatible with
executor progress and `Send`; an async-aware guard may legally cross when the
invariant requires it — evaluate invariant duration, contention, cancellation,
reentrancy, priority inversion, and whether state can be transactionalized
before suspension.

## Atomic doctrine

Atomics are a memory-ordering protocol, not faster variables. Every
non-trivial atomic algorithm documents:

```text
Published data:
Publishing operation/order:
Observing operation/order:
Happens-before edge:
Modification order assumptions:
ABA/reclamation handling:
Why Relaxed/Acquire/Release/AcqRel/SeqCst is sufficient:
```

Use the weakest correct ordering. `Relaxed` for independent counters or where
synchronization is established elsewhere; Release publishes, Acquire observes;
CAS success and failure orderings differ; `SeqCst` is not a substitute for
understanding. Benchmark contention and cache traffic, not uncontended
nanoseconds.

## Reclamation and ABA

Lock-free lookup is half the design: removed memory cannot be freed while
readers may access it. Policies: epoch-based reclamation, hazard pointers,
reference counting, immutable snapshots, deferred retire lists, arena epochs.
The public guard/closure API prevents references outliving protection:

```rust
shared.with(&key, |value| {
    // the borrow cannot escape the guarded epoch/lock
});
```

Bound memory retained by stalled readers. Address ABA with tagged/versioned
pointers, generation counters, reclamation that prevents observed reuse, or
ABA-insensitive designs — never "the allocator probably won't reuse it."

## Backoff, spinning, channels, parallelism

Short contention: bounded adaptive spin → yield → park. Never spin
indefinitely; a spinlock is disastrous when the owner can be descheduled or
awaits. Unbounded queues convert overload into memory growth and latency
collapse — bounded is the default when producers can outrun consumers; the
contract covers capacity units, full-queue behavior, fairness, batch ops,
close semantics, cancellation, wakeup strategy. Channel flavors by semantics:
MPSC for single-consumer message ownership, broadcast only when every
subscriber must see every value with defined lag, watch for latest-value
observation, oneshot for exactly-one-result handoff. Parallelize only above a
benchmark-derived work threshold; coarse chunks, independent writes, local
aggregation — no atomics in the inner loop.

## Async: capability-first, runtime-second

`core` and low-level engines describe capabilities (spawn/join, timers, I/O,
cancellation, blocking isolation, channels) without forcing Tokio or any
runtime into public semantics; runtime specifics live in adapters. No engine
starts invisible immortal background tasks without an explicit lifecycle
owner. Blocking or CPU-heavy work never runs on latency-sensitive executor
threads — route through blocking/CPU isolation. CPU-heavy async loops either
move to a CPU pool or yield at a measured interval (per-item yields are too
expensive; zero yields starve the runtime).

## The async state-machine budget

An `async fn` compiles to a state structure holding values live across
suspension. For hot/high-cardinality tasks: inspect which values survive each
`.await`; drop/scope large buffers, guards, and temporaries before suspension
(async prelude extraction — do the heavy synchronous preparation before the
future or in a helper whose temporaries die first); measure representative
future size in a targeted regression test; box only at dynamic/code-size
boundaries, never as reflexive "optimization." Fewer source lines across
`.await` does not mean a smaller future.

## `Send` profiles

Public async APIs are `Send` when the intended topology migrates tasks.
Thread-per-core/local runtimes may deliberately expose named `!Send` local
profiles for lower synchronization and richer borrowing. Never accidentally
make the only API local, and never force `Send` through unsafe assertions.

## Cancellation is a state transition

Every await point is a possible drop point. For mutations spanning
suspension: commit atomically after all fallible work or use a rollback
guard; never leave externally visible half-state; define ownership of queued
messages/resources on cancellation; make retries idempotent or assign
operation identity; bound shutdown and drain. Any future participating in
race/select must document whether cancellation can lose state or partially
consume input — and test it.

## Model checking

When gates are requested for custom synchronization, model-check the small
algorithmic core (Loom/Shuttle-class, bounded state). The requirement is
tool-neutral; do not expect a model checker to validate a whole application.
