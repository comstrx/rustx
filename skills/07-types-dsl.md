# Types, DSL, and API Design

**Load when:** designing RustX types, fluent pipelines, builders, borrowing
facades, conversions, callbacks, or polymorphism.

**Output:** a canonical, ownership-honest API whose common path is easy and
whose expensive paths are explicit.

## The ownership grammar (Prime Law 25)

Every public type and method belongs to exactly one of four categories, and
its name/signature declares which:

```text
Values = eager + owned          Str, List<T>, JsonOwned — construction pays now
Views  = borrowed + zero-copy   StrView<'a>, ListView<'a, T>, JsonRef<'a>
Pipes  = lazy + fused           iterator/pipeline stages — no work until driven
Result = materialized on demand collect/build/finish/to_* — the explicit moment cost is paid
```

No value pretends to be a view; no pipe materializes silently; every
materialization point is visible at the call site. This is the architectural
spine of the DSL — the rest of this file is its elaboration.

## Emulate fluency, not dynamism

RustX feels like Python in readability and composability — never in runtime
typing or hidden allocation:

```rust
let names = users
    .view()
    .filter(|user| user.active())
    .map(User::name)
    .take(100)
    .collect::<List<_>>();
```

Pipelines compile to fused iterators/static plans; intermediate collections
exist only by explicit materialization.

## Semantic type families, explicit profiles

A RustX "type" is a semantic family, not one physical layout:

```text
Str / StrView<'a> / SharedStr / StrBuilder
List<T> / SmallList<T, N> / FixedList<T, N> / ArenaList<'a, T> / ListView<'a, T> / SharedList<T>
Dict<K, V> / SmallDict<K, V, N> / SharedDict<K, V> / FrozenDict<K, V> / Cache<K, V>
JsonRef<'a> / JsonOwned / JsonTape<'a> / JsonArena
```

Never one mega-enum that pays tags, branches, and dynamic ownership
everywhere. `Cache` is not a map alias (TTL, admission, eviction, weight,
stampede control, loader cancellation, metrics are semantics above the storage
map). A concurrent backend is never the default local `Dict`. Hashing is a
policy under `Dict` — `SecureRandomized` for hostile keys, `FastTrusted`,
`Deterministic`, `Prehashed`, `Identity` — selected by threat model and
workload, never a public dependency choice.

## The DSL cost grammar

One vocabulary across every crate — no synonyms:

| Verb/prefix                     | Contract                                           |
| ------------------------------- | -------------------------------------------------- |
| `as_*`, `view`, `borrow`, `get` | borrowed view; no allocation under normal contract |
| `iter`, `stream`                | lazy traversal                                     |
| `map`, `filter`, `scan`         | lazy unless the type says materialized             |
| `collect`, `build`, `finish`    | materialize/commit                                 |
| `to_*`                          | create a new representation; may allocate          |
| `into_*`                        | consume source; reuse storage when possible        |
| `*_in`, `*_into`                | caller-provided storage/arena                      |
| `clone`, `snapshot`             | explicit duplication/shared snapshot               |
| `share`                         | shared-ownership transition                        |
| `freeze`                        | mutable→immutable phase transition                 |
| `prepare`, `compile`            | setup amortized over reuse                         |
| `parse`                         | validate and convert                               |
| `*_unchecked`                   | unsafe/trusted precondition                        |
| `entry`                         | one lookup plus mutation intent                    |
| `with`                          | scoped access whose guard cannot escape            |

A method called `view` never allocates behind the caller's back. Encode the
costs that alter architectural expectations — not every micro-cost.

## Ownership compression ladder

Prefer, in order: (1) ordinary borrowing with short scopes; (2) borrowed view
types; (3) consume-and-return/reuse APIs; (4) handles/indices into stable
owners; (5) scoped closure access hiding a guard; (6) `Cow` when
clone-on-write is genuine (not when the common path always clones); (7) `Arc`
when independent lifetime is required; (8) type erasure only at a real dynamic
boundary. The DSL makes the correct lifetime path obvious — it does not make
every value owned.

GAT-based lending traits serve iterators/views whose items borrow from
`self`:

```rust
pub trait LendingIterator {
    type Item<'a>
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>>;
}
```

## Typestate and builders

Encode protocol phases where misuse is meaningful and common:

```rust
let parser = Parser::<Unconfigured>::new().limits(...).build()?; // → Parser<Ready>
```

Typestate removes invalid calls and runtime branches — but never at the cost
of an unreadable generic forest; hide states behind aliases/facades. Builder
split: newtypes/typestate reject cheap local invalid states early; builders
collect cross-dependent configuration; `.build()` validates global rules
exactly once; hot methods never repeat construction checks. Builders store
plain values, avoid heap unless the data requires it, are `#[must_use]`, and
do no expensive work per setter. Use a builder only when it materially
improves construction — not by habit.

## Polymorphism escalation

```text
concrete/inherent
→ generic function or trait bound
→ enum of known strategies
→ sealed extension point
→ dyn Trait behind a RustX wrapper
→ FFI/plugin boundary
```

Essential functionality is inherent — traits model shared capability, not
basic discoverability. Escalate only when the lower level cannot represent
required heterogeneity; record code-size cost for heavily monomorphized
public APIs. Associated type when each impl has one output type; generic
parameter when many. Respect coherence: wrap foreign types in newtypes for
foreign traits. Keep traits dyn-compatible when `dyn` use is intended.

## Conversion discipline

Broad `Into`/`AsRef` bounds hide allocation, increase monomorphization, and
create inference ambiguity. Use them at ergonomic boundaries, then normalize
immediately to a narrow internal representation; hot internals take concrete
views (`&[u8]`, `StrView<'_>`). `From` (never bare `Into`) for infallible
conversions; `TryFrom` for narrowing/fallible; `FromStr` for string parsing —
but never force a borrowed parser through `FromStr` (it cannot tie output
lifetime to input; use an inherent `parse` whose signature shows the borrow).
Accept `&[T]`/`&str`, not `&Vec<T>`/`&String`. `as` never narrows — pedantic
cast lints are denied workspace-wide.

## `Str`, `Num`, `Func`

- **Str:** the owned type and borrowed views are separate; a lifetime-free
  `Str` cannot borrow arbitrary temporaries without allocating, refcounting,
  interning, or lying — hide syntax complexity, never violate the fact. Text
  engines are byte-first: operate on `[u8]`, validate UTF-8 in blocks, convert
  at the narrowest boundary. Every length/slice/case API states its unit:
  bytes, scalars, graphemes, columns.
- **Num:** static numeric types stay; `Num` provides traits, parsing/formatting
  engines, checked/wrapping/saturating/strict policies, and explicit
  `Strict`/`Algebraic` float modes. Overflow vocabulary is explicit; build
  profile never silently decides domain correctness. `NonZero*` for niches;
  `total_cmp`/tolerance for float comparison; floats as `Dict` keys demand an
  explicit NaN/zero/hash policy.
- **Func:** hot callbacks are generic `F: FnMut(...)` (least restrictive Fn
  trait, `impl Fn` returns, no `Box<dyn Fn>` reflexes). An erased `Func` type
  is justified for heterogeneous storage, plugin registries, ABI boundaries,
  or measured code-size control.

## Public surface and AI legibility

One canonical path per public item; curated root re-exports; no public glob
re-exports; no mirrored prelude/root/module ambiguity; concrete names; errors
that point to a narrow correction. Third-party types stay private — explicit
`native()`/`into_native()` escape hatches where interop is a contract. Public
types state intended auto-traits (`Send`/`Sync`/`Unpin`) and cloning cost.
`#[non_exhaustive]` is workspace policy on public enums/structs (lint-enforced)
— design match ergonomics accordingly.

## Macros in the DSL

Macros compress syntax, not semantics. Ladder: function/trait/builder →
`macro_rules!` → derive/attribute proc macro → external codegen. Good uses:
literals (`dict!`, `json!`) with clear ownership, schema generation, static
tables, compile-time validation. Never: hidden allocation or I/O, invisible
control flow, swallowed errors, manufactured unsafe, giant expansions for tiny
convenience. Hygiene: `$crate` paths, precise fragment specifiers (not raw
`:tt`), third-party names through a `#[doc(hidden)] pub mod _private`. Proc
macros live in a thin `proc-macro = true` crate; semantic logic lives in a
normal crate with ordinary tests; errors are spanned compile errors, never
panics. Generated diagnostics are part of DSL quality.

## Async in the DSL

Never hide an await-worthy operation behind a synchronous-looking method.
Async boundaries, blocking work, cancellation, and backpressure stay visible.
Runtime-neutral core traits avoid hard-coding an executor; public `async fn`
in traits only where the future bounds are sufficient and controlled — public
runtime-facing traits may need an explicit future-returning contract to let
callers demand `Send`. See [09-concurrency.md](09-concurrency.md).

## Naming (beyond the cost grammar)

Rust conventions absolutely: `UpperCamelCase` types/variants, `snake_case`
functions, `SCREAMING_SNAKE_CASE` consts, acronyms as words (`HttpServer`),
no `get_` prefix on getters, `is_`/`has_`/`can_` booleans,
`iter`/`iter_mut`/`into_iter`, iterator types named after their source method,
short conventional lifetimes (`'a`, `'de`, `'src`), descriptive generics once
multiple parameters have distinct roles, no `-rs` crate suffix.
