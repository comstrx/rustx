# Source Adaptations — Audit Archive

What external guidance changed on its way into the constitution, and why.
This is an audit trail, not an authority override. A rule is rejected only
after identifying the invariant it protects; dependency-specific wording is
normally **adapted**, never discarded; the reason is recorded rather than
replaced with taste.

## Agent Skills standard

**Adopted:** standards-compliant `SKILL.md` frontmatter; progressive
disclosure (compact orchestrator + focused routed files); intent-focused
activation description; behavioral evals comparing with-skill vs baseline;
deterministic validators over re-invented mechanical checks.

**Strengthened for RustX:** context consumption is a performance metric;
external prose sits behind an instruction firewall and source lock; routing
is by technical decision signal, not topic names; evals assert architectural
behavior and proof honesty, not wording.

## Microsoft Pragmatic Rust Guidelines 2026.6

**Adopted nearly directly:** AI/human API legibility; Rust-shaped solutions;
one canonical public path; end-state documentation; no tautological tests;
async future size as a budget; allocation-reuse APIs and build-then-freeze;
selective shrinking; no public glob re-exports; mockable system boundaries;
macro hierarchy with thin proc-macro impl crates; additive features; hermetic
builds; workspace centralization; panic continuation only at containment
boundaries; FFI translation crates and dylib state isolation; structured
telemetry with secret redaction; adversarial hardening of unsafe abstractions
(poison on closure panic; assume misbehaving safe traits).

**Adapted:**

| Upstream spirit                        | RustX rule                                                                             |
| -------------------------------------- | -------------------------------------------------------------------------------------- |
| app errors may use anyhow-class crates | all crates speak `core::error`; dynamic reporting only at the outermost edge           |
| use mimalloc for applications          | allocator is a measured workload decision, never a blanket dependency                  |
| async fns are a friendly default       | async is an I/O/topology boundary with size, `Send`, and cancellation budgets          |
| public types should be `Send`          | `Send` where migration is intended; named `!Send` local profiles allowed               |
| shrink after building                  | freeze/shrink only when lifetime, slack, movement, and regrowth make it a net win      |
| fast hashing for performance           | only for trusted keys or bounded collision impact; hostile keys get protected profiles |
| always run broad static verification   | compilation is continuous; formal gates are explicit On-Demand checkpoints             |
| `target-cpu` highest viable for apps   | controlled binaries only; portable libraries keep baseline + dispatch                  |

**Rejected (with the protected invariant named):**

- **M-NO-PRELUDE** ("crates must not define preludes"). It protects namespace
  hygiene when _unrelated vendors'_ preludes collide. RustX preludes are a
  single-vendor, curated, collision-free vocabulary — the DSL's front door —
  and the prelude policy in [12-anatomy.md](12-anatomy.md) plus `conform`
  guard the actual hazard. The constitution wins; the hazard stays named.

## `leonardomso/rust-skills` (MIT) via the rustx-skills pack

All 265 rules reviewed rule-by-rule upstream: 138 KEEP / 112 ADAPT /
15 MERGE / 0 hard rejects — full table in [\_rule-matrix.md](_rule-matrix.md),
distilled canon in [18-canon.md](18-canon.md). The twelve high-impact
reinterpretations: own the error contract (not thiserror/anyhow); memory
helpers become RustX engines (not SmallVec-as-contract); async is
capability-first (Tokio in adapters); performance-first is not folklore-first
(structural performance early, evidence for claims); SIMD is a layer;
hashing lives under `Dict`; iterators are not a religion; Serde is
compatibility not architecture; testing tools are replaceable capabilities;
lints/gates obey On-Demand; modern module layout; the DSL cannot conceal
expensive semantics.

Upstream corrections carried: `collect_into` unstable; async-fn-in-trait
public `Send`-bound caveat; blocking vs async-aware locks across `.await`;
`mod.rs` not required.

## This forge's own verdicts (2026-08-22)

- **Module style flipped to modern `foo.rs` + `foo/`** and enforced by lint
  (`clippy::mod_module_files = deny` replacing `self_named_module_files`).
  Grounds: both source packs, ecosystem standard, and agent legibility
  (unique filenames); zero migration cost at flip time. This also resolved
  rustx-prime's internal contradiction (its text said modern; its diagrams
  said `mod.rs`).
- **Whole-group pedantic/nursery/cargo denial kept** over upstream
  "selective" advice — stricter, proven green, `#[expect(reason)]` is the
  escape.
- **No unwrap/expect anywhere in library code kept** over upstream
  "expect for bugs" — invariants use assertions with messages, types, or
  structured errors.
- **Workspace-external `tests/`/`benches/`/`examples/`/`bloats/` kept** over
  per-crate directories — published crates carry no dev-deps and contract
  tests link like consumers; enforced by `conform`.
- **1.98 ledger corrected** against the compiler: `substr_range` returns
  `core::range::Range`; `trim-paths` unstable; the rest verified as claimed.
