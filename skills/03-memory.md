# Memory, Layout, and Allocation

**Load when:** designing arenas, buffers, pools, ownership phases, immutable
storage, or data layout.

**Output:** a lifetime geometry and byte-journey showing why the chosen
representation minimizes total movement.

## Memory is topology, not allocation count

The dominant questions: where does data live, who owns it, how long does it
live, how many pointers must be followed, which cores mutate which cache
lines, can values move, can destruction be bulk or deferred?

## Lifetime geometry

Classify memory by phase before choosing a container:

| Phase                       | Typical policy                                                                        |
| --------------------------- | ------------------------------------------------------------------------------------- |
| expression / tiny temporary | stack or inline storage                                                               |
| operation scratch           | caller-reused buffer or resettable scratch arena                                      |
| request / transaction       | bounded owner with deterministic teardown                                             |
| parsed document             | input borrow + document arena / compact tape                                          |
| immutable service snapshot  | build mutable → validate → freeze into shared storage                                 |
| long-lived mutable service  | explicit capacity and reclamation policy                                              |
| global process lifetime     | immutable constants or deliberately leaked tables only — never correctness singletons |

The key optimization is often **phase conversion**: mutable build
representation → validated frozen representation → read-only shared view.

## Allocation hierarchy

Prefer the cheapest lifetime-compatible storage:

```text
borrowed slice/reference
→ stack / inline capacity
→ caller-provided scratch buffer
→ thread-local reusable buffer
→ phase arena / bump allocation
→ pooled heap allocation
→ general heap allocation
→ shared reference-counted allocation
```

Not absolute: large stack objects, pathological inline enums, and global pools
can be worse than the heap. Measure real sizes and lifetimes.

## Allocation reuse as API design

Offer reuse without infecting the simple path — names expose ownership
geometry:

```rust
fn parse(input: &[u8]) -> Result<Document>;
fn parse_into<'a>(input: &[u8], out: &'a mut DocumentBuf) -> Result<DocumentView<'a>>;
fn parse_in<'a>(input: &'a [u8], arena: &'a Arena) -> Result<DocumentRef<'a>>;
```

Reuse APIs preserve capacity via `.clear()`/reset while proving stale handles
cannot escape. Also: `clone_from()` to reuse allocations on repeated clones;
clear-and-reuse collections in loops; `mem::take`/`mem::replace` to move out of
`&mut` without cloning; `with_capacity()` when size is known; `Extend::extend`
for batch reuse (`Iterator::collect_into` is still unstable on 1.98 — see
[15-rust-198.md](15-rust-198.md)).

## Build → freeze

For large, long-lived immutable data evaluate: `Vec<T> → Box<[T]>` (drops
retained capacity, shrinks the handle), `String → Box<str>`,
`String/Box<str> → Arc<str>` for shared immutable text, compact offset tables
plus one payload slab for many small strings, interning only when equality
reuse dominates hashing/contention/retention cost. Freezing is a semantic
transition: it disables mutation in the type and states whether handles remain
stable.

`shrink_to_fit` is a request to the allocator, can move memory, and may leave
spare capacity. Use it only when construction has ended, the collection is
long-lived, retained slack is material, regrowth is unlikely, and the latency
spike is off the hot path. Otherwise retained capacity is a reuse asset.

## Arena profiles

`Arena` never means one universal allocator. Distinct semantic profiles:

- **Scratch** — mark/rewind lifetime; nothing escapes the scope; per-worker by
  default; rollback on error/cancellation.
- **Bump document** — many allocations, bulk release, stable addresses within
  arena lifetime; explicit destructor policy.
- **Drop-aware** — records values requiring `Drop`; destruction paid at
  reset/drop; bulk-free is not free when destructors matter.
- **Generational** — stable handles + generation counters reject stale handles;
  for mutable graphs, slot reuse, registries.
- **Frozen** — mutable construction phase, immutable shared read phase.

A shared concurrent bump pointer is never the default: per-worker arenas remove
contention and improve locality.

Every arena implementation defines: alignment behavior, integer-overflow
checks, chunk growth, maximum allocation behavior, ZST behavior, reset/rewind
semantics, destructor semantics, address stability, `Send`/`Sync` conditions,
panic/cancellation behavior, stale-handle behavior.

## Buffer families

`Buffer` is a semantic facade over purpose-built storage: `Buffer` (contiguous
mutable bytes with cursors), `BufferView<'a>`, `SharedBuffer` (cheap immutable
slicing), `RingBuffer`, `SegmentedBuffer` (no large memmoves, vectored I/O),
`ScratchBuffer`, `FixedBuffer<N>` (inline capacity, explicit overflow policy).
A common facade can expose distinct concrete types — never one branch-heavy
enum unless benchmarks prove the branch and size costs negligible.

Buffer rules: separate `len` from `capacity` and read cursor from write
cursor; reserve from protocol knowledge; reuse capacity when retention is
bounded; compact only when the dead prefix materially hurts; expose
`IoSlice`/vectored boundaries where the OS can consume segments; **never**
expose references to uninitialized capacity; **never** `set_len` before every
included element is valid.

## Correct partial initialization

For high-throughput writes into spare capacity:

1. Reserve once.
2. Obtain `spare_capacity_mut`.
3. Write with `MaybeUninit::write`.
4. Track the initialized count in a panic-safe guard.
5. Commit `len` only to initialized elements.
6. On unwind, drop exactly the initialized prefix — never more, never less.

Reference blueprint (adapt, don't copy blindly):

```rust
struct SpareInitGuard<'a, T> {
    spare: &'a mut [MaybeUninit<T>],
    initialized: usize,
    committed: bool,
}

impl<T> Drop for SpareInitGuard<'_, T> {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        for slot in &mut self.spare[..self.initialized] {
            // SAFETY: the writer initializes the prefix exactly once and
            // increments `initialized` only after each write succeeds.
            unsafe { slot.assume_init_drop() }
        }
    }
}
```

A sound initialization guard outranks eliminating one branch. If an API
intentionally preserves the initialized prefix after a panic, that is a
different named contract with a set-length-on-drop guard — panic behavior is
never an accident of implementation.

## Data layout doctrine

- **AoS** when most fields of an item are consumed together; **SoA** when hot
  loops consume few fields across many items (denser cache lines, easier SIMD).
- **Hot/cold split**: move rarely-read metadata, strings, errors away from the
  hot record.
- **Indices vs pointers**: compact indices reduce record size, serialize, and
  avoid provenance complexity; pointers avoid lookup. Measure both.
- **Niche optimization**: `NonZero*`, references, and suitable enums enable
  compact `Option` layouts — verify with layout assertions; never promise ABI
  from unspecified `repr(Rust)`.
- **Alignment** only for a measured reason (SIMD loads, DMA/FFI, false-sharing
  isolation). Static-assert type sizes to guard accidental growth.
- **Pointer-depth budget**: hot fields reachable with the fewest dependent
  loads; lift hot metadata out of nested boxes/Arcs/trait objects.
- Box an enum variant only when size disparity materially harms layout and the
  indirection is worth it; choose integer widths from domain range and
  whole-structure layout economics, not reflex.

## False sharing

Independent atomics sharing a cache line serialize cores. For hot per-core
state: shard by worker, pad only the contended fields, aggregate locally and
merge in batches. Do not blanket-align every type to 64 bytes.

## Pooling

Pools work when allocation cost and size classes are stable; they are
dangerous when they retain peak memory forever or centralize contention. A
pool defines: ownership and return protocol, maximum retained capacity, size
classes, cross-thread behavior, poisoning/reset behavior, sensitive-data
clearing, shutdown semantics.

## The byte-journey audit

For every hot operation, draw the journey and mark every copy, allocation,
ownership conversion, cache-unfriendly hop:

```text
source → validation → normalization → staging → engine → result → serialization
```

Remove redundant journeys before tuning instructions. A strategic copy is
valid when it buys contiguity, alignment, smaller elements, or repeated-query
reuse.

## The stable allocator boundary (1.98)

Rust 1.98 does not provide the generic `Allocator` API stably for `Box`, `Arc`,
and collections. Own arenas and buffers directly; use the global allocator
boundary; quarantine allocator-generic experiments in `nightly-lab`; never
expose unstable allocator types in public contracts.

Also: know drop order (struct fields top-to-bottom, locals in reverse) and
control it when teardown matters.
