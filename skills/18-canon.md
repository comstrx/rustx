# The One-Line Rule Canon

**Load when:** reviewing code quickly, or as a checklist sweep after a round.
Distilled from a rule-by-rule audit of 265 upstream rules (KEEP/ADAPT/MERGE →
250 canonical), rewritten for RustX. Deeper doctrine lives in the routed
files; workspace lints already enforce many of these mechanically.

## Ownership

- Borrow (`&T`) over `.clone()`; accept `&[T]`/`&str`, never `&Vec<T>`/`&String`.
- `Cow<'a, T>` only for genuine conditional ownership — not when the common path always clones.
- `Arc`/`Rc` only for genuinely shared ownership; prefer borrowing, handles, arenas, transfer.
- `RefCell` only where runtime borrow checks are an accepted trade-off (workspace bans it in libs — use `Cell`, domain state, or an encapsulated `UnsafeCell` capsule).
- `Copy` only for semantically cheap trivially duplicated values; explicit `Clone` where copying costs.
- Indirection (`Box`, handles) for layout/stable-address reasons or measured move cost — not "it's large."
- Rely on lifetime elision; explicit lifetimes only when required.

## Errors

- `Result` for recoverable failures; the unified error vocabulary at every boundary.
- No `unwrap`/`expect` in library code (lint-enforced); invariants use `assert!` with messages or types.
- `?` for propagation; `From<E>` for conversions — not scattered `map_err`.
- Preserve source chains; never flatten a chain into a string.
- Messages lowercase, no trailing punctuation; `# Errors` documented.

## Memory

- `with_capacity` when size is known; reserve from protocol knowledge.
- Reuse: `clone_from`, clear-and-reuse in loops, `drain`, `extend`, `mem::take`/`replace`.
- `write!` into buffers over `format!` allocations; no `format!` where a literal works.
- Inline-first/fixed-capacity storage when benchmarks show allocation wins — behind RustX types.
- Box an enum variant only when size disparity measurably harms layout.
- Static-assert type sizes; know and control drop order.

## Unsafe

- `// SAFETY:` on every block (lint-enforced), `# Safety` on every unsafe fn; smallest possible scope.
- `MaybeUninit` for uninitialized memory — never `mem::zeroed` for validity-bearing types.
- Rust 2024 forms: `unsafe extern { }`, `#[unsafe(no_mangle)]`.
- Manual `Send`/`Sync` requires a written state-by-state proof; prefer compiler derivation.
- Miri-compatible tests exist for meaningful unsafe; run at gate checkpoints.

## API

- Builders `#[must_use]`, validate in `.build()`; builder only when it materially helps.
- Newtypes prevent mixing semantically different values and guard invariants at construction.
- Parse, don't validate: boundaries produce validated types.
- Sealed traits control implementation; extension traits add fluency.
- `#[must_use]` where ignoring a result is likely a bug; `#[non_exhaustive]` is workspace policy.
- `From` not bare `Into`; `TryFrom` for narrowing; `FromStr` for owned string parsing only.
- Implement `Default`, `Debug`, `Clone`, `PartialEq` etc. where semantics allow; `FromIterator`/`Extend`/`IntoIterator` (all three reference forms) for collections.
- Operators only with natural, unsurprising semantics.

## Async

- Runtime-neutral semantics; Tokio/executors live in adapters.
- No synchronous guard across `.await`; async-aware guards deliberately and narrowly.
- Blocking/CPU-heavy work isolated off executor threads; async-aware fs/io inside async paths.
- Long-lived operations cancellable; cancellation safety documented and tested for race/select.
- Bounded channels by default; MPSC/broadcast/watch/oneshot chosen by exact semantics.
- Structured task ownership: join, cancel, drain deterministically.
- Clone cheap handles before `move` when borrows are fragile — not ref-count churn in hot loops.
- `async fn` in traits where future bounds suffice; explicit future contracts where callers need `Send`.

## Concurrency

- Data parallelism only above the amortization threshold.
- `std::thread::scope` to borrow across threads; `thread_local!` over `static mut`.
- Weakest correct atomic `Ordering`, documented.

## Types

- Newtype IDs (`UserId(u64)`); enums for exclusive states; `Option`/`Result` — never sentinels.
- No stringly-typed APIs. `PhantomData` for type relationships; `!` for divergence.
- Bounds only where needed; `where` clauses for readability.
- `repr(transparent)` for FFI newtypes; `Deref` only for smart pointers/transparent wrappers.
- `Display` for users, `Debug` for diagnostics; hex/octal/binary impls for numeric newtypes.

## Numerics

- Overflow explicit: `checked_`/`saturating_`/`wrapping_`/`overflowing_`.
- Widen with `From`, narrow with `TryFrom` — never `as` (lint-enforced).
- Float compare via tolerance/ULP/`total_cmp`; `clamp` to bound; `NonZero*` for niches.

## Const

- `const { }` blocks for compile-time assertions; `const fn` where possible.
- Const generics for value parameters; `const` = inlined value, `static` = single address.

## Patterns

- `let ... else` for early-return extraction; `matches!` for boolean tests; if-let chains; `@` bindings.
- Match RustX-owned closed enums exhaustively (lint-enforced) so new variants force review.

## Macros

- A macro only when a function or generic cannot express it (see [12-anatomy.md](12-anatomy.md)).

## Collections

- `BinaryHeap` for priority; sets for membership — not linear `Vec::contains`.
- Map/sequence semantics chosen by access pattern behind RustX facades; entry API for insert-or-update.

## Naming

- Rust casing conventions; `as_`/`to_`/`into_` cost prefixes; `is_`/`has_` booleans; no `get_`; acronyms as words; iterator types named after their methods; no `-rs` suffix.

## Testing

- Unit tests in `#[cfg(test)] mod tests`; integration via the workspace `tests/` crate.
- Descriptive names; property tests for invariants and round-trips; fakes over mock frameworks.
- `#[should_panic]` only for APIs whose documented contract panics (rare here).
- Doctests are executable; benchmarks use `black_box`; snapshots only for stable large outputs.

## Documentation

- Every public item documented (lint-enforced); `//!` module docs; examples with `?`; intra-doc links; full crate metadata.

## Observability

- Structured fields, not interpolated strings; libraries emit, applications subscribe; log an error once with its chain; never secrets/PII.

## Performance

- Lazy iterators, collect once, no intermediate collects; entry/drain/extend.
- The clearest zero-overhead form until codegen or benchmarks justify indexing/chunking/SIMD.
- Structural performance designed early; claims only from evidence.
- Buffered/batched I/O by measurement.

## Project

- `main.rs` minimal, logic in `lib.rs`; multiple binaries in `src/bin/`.
- Modern `foo.rs` + `foo/` module layout (lint-enforced); canonical anatomy everywhere.
- `pub(crate)`/`pub(super)` internal visibility; curated `pub use` re-exports; small deliberate preludes.
- Workspace-centralized versions, metadata, lints, profiles; additive features; declared MSRV; minimal deterministic `build.rs`.

## Gates

- The lint wall lives in the root `Cargo.toml` and `.clippy.toml` — stricter than any upstream advice (whole pedantic/nursery groups denied; `#[expect(reason)]` is the only escape).
- Compilation is continuous and mandatory; every other gate is On-Demand via `bash scripts/run.sh <gate>`.
