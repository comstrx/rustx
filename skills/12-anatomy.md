# Crate Anatomy, Public APIs, Docs, Macros, and Tests

**Load when:** creating crates, modules, exports, preludes, docs, macros, or
test strategy.

**Output:** one canonical public API with honest costs, end-state docs, and
independent contract tests.

## Canonical crate shape

`bash scripts/run.sh new <name>` scaffolds it; `conform` (inside `ci-lint`)
enforces it. Use only the files the crate needs; keep names and
responsibilities uniform:

```text
crates/<name>/            # package name: rustx-<name>
├── Cargo.toml            # inherits every workspace field + lint policy
├── LICENSE-MIT / LICENSE-APACHE
└── src/
    ├── lib.rs            # crate docs, module declarations, public re-exports
    ├── prelude.rs        # the crate's deliberate vocabulary
    ├── facade.rs         # ergonomic public entry points (when warranted)
    ├── error.rs          # domain mapping into the core error vocabulary
    ├── constants.rs      # measured/derived constants with provenance
    ├── <concept>.rs      # one file per concept
    └── engine/           # private physical implementations (kernels, dispatch)
```

**Module style is modern `foo.rs` + `foo/`** — never `mod.rs` (lint-enforced:
`clippy::mod_module_files` is denied). Unique filenames navigate better for
humans and agents alike.

Support code lives **outside** published crates in the workspace-level
`tests/`, `benches/`, `examples/`, `bloats/` member crates: published crates
carry zero dev-dependencies, and contract tests can only reach the public API
— API-first is enforced by layout, not discipline. Per-crate unit tests still
live inline in `#[cfg(test)] mod tests`.

## Facade vs engine

Facade owns ergonomic names, defaults, builders, options, conversions, error
context. Engine owns representation, kernels, dispatch, unsafe, backend
strategy, hot loops. The engine never depends on facade formatting or I/O.

## Canonical public surface

- One canonical path per public item. Root re-exports are curated; module
  paths are either public namespaces or private — never both competing.
- **No glob re-exports in the public surface** (the prelude is the deliberate
  exception below).
- Essential operations are inherent; traits model shared capability.
- Third-party types stay private; explicit `native()`/`into_native()` escape
  hatches where interop is a contract.
- Public types state intended auto-traits and cloning cost where it matters.
- `pub(crate)` for internals, `pub(super)` for parent-only visibility.
- `lib.rs` describes the public architecture; it contains no implementation.

## Prelude policy

Each crate ships a small deliberate `prelude`; the facade crate's prelude is
the union of member preludes. A prelude **must not** export everything, cause
method collisions, hide heavyweight optional dependencies, or make unsafe
APIs casually available.

> Recorded exception to Microsoft M-NO-PRELUDE ("don't define preludes"):
> that guideline protects namespace hygiene across _unrelated_ vendors'
> preludes. RustX preludes are a single-vendor, curated, collision-free
> vocabulary — the DSL's front door — and `conform` plus this policy guard
> the actual hazard. The constitution wins; the hazard stays named.

## Documentation contract

Docs describe the **end state**, never the design journey (ADRs hold
history). Every public item:

1. a short semantic summary — first sentence one line, ~15 words;
2. cost/ownership behavior when non-obvious;
3. `# Examples` for the common path — runnable doctests, `?` not `unwrap`,
   `#`-hidden setup lines;
4. `# Errors`, `# Panics`, `# Safety` sections when applicable;
5. invariants and profile differences;
6. no claims lacking linked evidence.

Module docs (`//!`) explain vocabulary and selection — they do not repeat
items. Intra-doc links connect related items. `#[doc(inline)]` on `pub use`
items that should render in place. Keep crate docs and README from drifting
(`include_str!` when the layout fits). Fill full `Cargo.toml` metadata for
published crates. Missing docs are denied workspace-wide — write them with
the code, not after.

## Macro policy

Ladder: function/trait/builder → `macro_rules!` → derive/attribute proc macro
→ external codegen. A macro never makes code look like a function while
changing evaluation count, control flow, or item shape. Declarative macros:
`$crate` paths, precise fragment specifiers, third-party expansion names via
`#[doc(hidden)] pub mod _private`, `#[macro_export]` with a clean import
path. Proc macros: dedicated `proc-macro = true` crate as a thin
parser/validator/emitter; semantic logic in a normal crate with ordinary
tests; spanned compile errors, never panics; no implied or hidden items;
`syn`/`quote` only when their leverage justifies the compile cost.

## Testing doctrine — contracts, not source text

Reject tautological tests: asserting a constant equals itself, mirroring
match arms, duplicating the implementation algorithm. Prefer:

- independent scalar/model oracles and differential tests;
- round-trip and metamorphic properties (property testing — tool selectable);
- boundary and adversarial cases;
- public integration tests through canonical paths (the workspace `tests/`
  crate links like a real consumer);
- compile-fail/trait-bound assertions for misuse;
- layout/future-size regression assertions where target policy makes them
  meaningful;
- descriptive test names; visually obvious arrange/act/assert; RAII cleanup.

Mock only behavioral boundaries with real signal; prefer small fakes; never
introduce abstraction only to satisfy tests (the production boundary must
also benefit). Test utilities ship feature-gated, never in default surface.
Async tests run through the crate's selected harness — `#[tokio::test]` only
inside Tokio-specific adapters.

## Lint exceptions and magic values

The only sanctioned escape is a narrow `#[expect(lint, reason = "…")]` whose
reason names the invariant or measured tradeoff (`allow` attributes are
denied workspace-wide; a fulfilled `expect` warns when it becomes stale).
Every threshold, capacity, spin count, shard count, and inline size is named
and carries provenance: derived formula, benchmark corpus, hardware
assumption, or safety bound. A number without origin becomes folklore.
