# Architecture — The Triangle Breaker and the Core Contract

**Load when:** the task spans multiple domains, changes `core`, primitive traits,
shared types, error identity, or dependency direction.

**Output:** a ranked architecture decision with explicit invariants, costs, and
proof obligations.

## The layered triangle

RustX does not "break" the simplicity–speed–safety triangle with one magical
abstraction. It places each concern in the layer where it is cheapest:

```text
┌──────────────────────────────────────────────────────────────┐
│ Facade / DSL                                                 │
│ terse, discoverable, Python-like, coherent, cost-transparent │
├──────────────────────────────────────────────────────────────┤
│ Static semantic plan                                         │
│ ownership, lifetimes, typestate, traits, policies, pipelines │
├──────────────────────────────────────────────────────────────┤
│ Engine                                                       │
│ data-oriented storage, arenas, buffers, hashing, parsing     │
├──────────────────────────────────────────────────────────────┤
│ Kernels                                                      │
│ scalar oracle + target-specialized SIMD/SWAR/atomic kernels  │
└──────────────────────────────────────────────────────────────┘
```

The facade removes cognitive noise. The type system removes illegal states.
The engine owns representation. The kernels own hardware specialization.

Complexity is compressed downward, never denied:

- invariants become types;
- ownership becomes phase-oriented APIs;
- specialization becomes private dispatch;
- unsafe becomes a proof-carrying capsule;
- setup becomes prepared state;
- diagnostics become cold data rendering;
- dynamic capability becomes an explicit profile.

The DSL **may hide**: guard types, arena handles, backend selection, CPU
dispatch, buffer growth, error plumbing, safe encapsulation of unsafe.
The DSL **must expose** (through names or types): allocation vs borrowing,
cloning vs sharing, blocking vs async, strict vs relaxed numerics,
deterministic vs randomized hashing, bounded vs unbounded resources,
validation vs trusted-input paths.

One semantic API, multiple physical engines: a universal representation is a
tax collector — every workload pays for capabilities only some need.

## Decision dominance hierarchy

When two designs compete, compare in this order; a lower-ranked win cannot
compensate for a higher-ranked failure unless the contract explicitly
reprioritizes:

1. semantic correctness and soundness;
2. adversarial robustness and failure containment;
3. workload fit and data movement;
4. tail behavior and memory footprint;
5. steady-state throughput/latency;
6. DSL clarity and misuse resistance;
7. compile cost, binary size, maintenance cost.

## Proof-carrying architecture

Choose a technique and its proof burden at the same time:

| Mechanism                  | Required companion                                        |
| -------------------------- | --------------------------------------------------------- |
| `unsafe`                   | safety case + safe boundary                               |
| SIMD / target feature      | scalar oracle + dispatch/fallback proof                   |
| lock-free reclamation      | memory-model argument + model tests                       |
| arena lifetime compression | phase/lifetime contract                                   |
| zero-copy view             | input lifetime + aliasing contract                        |
| external dependency        | admission dossier                                         |
| "fastest" claim            | benchmark dossier                                         |
| public dynamic dispatch    | runtime heterogeneity justification                       |
| panic continuation         | containment/restart contract                              |
| proc macro                 | grammar, diagnostics, expansion tests, regular impl crate |

## `core` is the intellectual ABI

`core` (the future substrate crate) is the shared language every RustX crate
speaks — primitive traits, unified errors, memory machinery, shared types:

```text
core
├── traits
├── error
├── memory   (arena · buffer · simd · scan · memmatch · hash · alloc)
└── types    (str · num · list · dict · json · func)
```

Internal dependency direction is acyclic and intentional:

```text
primitive traits / low-level error code
→ memory primitives and kernels
→ owned and borrowed types
→ richer context/reporting facades
```

The low-level allocator or SIMD layer never depends on a rich allocating error
formatter. "Intellectual ABI" means shared semantics and vocabulary — it is
**not** a promise that Rust layout is stable across compiler versions or
dynamic-library boundaries; FFI remains a separate translation boundary.

### `core` admission rule

A capability belongs in `core` only when **all** hold:

1. several higher crates need the same semantic vocabulary;
2. placing it lower removes real dependency inversion or duplicated policy;
3. it stays independent from I/O, runtime, platform services, business logic;
4. its public contract is stable enough to become workspace-wide infrastructure;
5. its compile/monomorphization footprint is acceptable for ubiquitous use.

### Primitive trait test

A trait belongs in `core` only if it names a RustX-wide capability (not one
implementation), has a small coherent required surface, forces no allocation,
runtime, or external types, states its object-safety or static-dispatch intent,
and has at least two credible consumers — or one foundational consumer with no
higher home. Use extension traits for fluent DSLs; seal traits whose invariants
RustX must control; never build a god trait, and never create a trait merely to
mock a concrete type or "future-proof" a single implementation.

### Error core inside `core`

The lowest error representation is compact and non-allocating:

```rust
#[repr(transparent)]
pub struct ErrorCode(NonZeroU32);
```

Rich context, chains, backtraces, and reports layer above it. The success path
never allocates to prepare for a possible error. See [08-errors.md](08-errors.md).

### `core` feature lanes

```text
default/stable  — Rust 1.98 production contract
std             — runtime detection, OS integration, richer diagnostics
alloc           — owned collections without full std
nightly-lab     — isolated experiments; never required by stable semantics
serde-interop   — compatibility only
```

Features are additive capabilities, never contradictory personalities.
`nightly-lab` is never a transitive requirement of normal crates.

## Dependency direction across crates

Lower crates never import higher vocabulary. Shared code moves downward only
when it is a genuinely general, stable abstraction — not to silence duplication
metrics. Deduplicate **knowledge and policy**, not lines: two similar functions
with different semantic contracts are not duplication. Every abstraction must
answer: what invariant does it centralize, what duplication does it remove,
what cost does it introduce?
