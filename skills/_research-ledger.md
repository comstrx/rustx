# Research Ledger — Audit Archive

Provenance for the RustX Engineering Constitution v1. Agents do not load this
file during normal tasks; it exists so every rule can be traced to its source,
review, and verification.

## 2026-08-22 — Constitution v1 forged

### Source materials reviewed (full personal read, file by file)

Source lock — SHA-256 of the exact reviewed artifacts:

```text
1cc2cf0b17273df87a16658871251d43edb4da4c6af57ed3580b07a3d4daabd5  rust skills/SKILL.md            (golden monolith, 3431 lines)
eba4fb4c2d8d8fa4f5a21f57b5b4b6648557cf73d679020d29c450440eec1e4d  rustx-prime/SKILL.md            (v2.0.0 orchestrator)
8f22766cb3e713d1f33554c7eac2dfd80f1966f3b89f23bf1c41b37e51e0ca85  rustx-prime/RUSTX_PRIME_COMPENDIUM.md
068137fcab874e6c5ef9e9f3cd7e00f6c8ad406869d5625ade91e912372012da  rustx-skills/SKILL.md           (265-rule pack, v1.0.0)
299ad45ce2b0229a94899c519328bc450e6f55573434744814aca28d37b2939d  rustx-skills/RULE_MATRIX.md
c9f1ac1c9d52a65e0c4d524e6a5c2eb80a33669b8b26b7537f0d66a555f432f7  _microsoft-guidelines-2026.6.snapshot.txt
```

Also read in full: rustx-prime references 00–19, assets (9 templates), evals,
CHANGELOG, RESEARCH_LEDGER, SOURCE_LOCK; rustx-skills rules 01–26, DECISIONS,
DEPENDENCY_POLICY, SOURCE_CORRECTIONS, WORKFLOW, ATTRIBUTION.

### Upstream provenance chains

- **rustx-prime 2.0.0** (researched 2026-08-21): synthesized from the Rust
  1.98 release ledger, Rust Reference/Rustonomicon/rustc/Cargo docs, the Agent
  Skills specification, Microsoft Pragmatic Rust Guidelines 2026.6 (official
  generated source dated 2026-08-19), the `ms-rust-skill` routing/hashing
  strategy, and the engine literature (memchr, simdjson + "Parsing Gigabytes
  of JSON per Second", simd-json, sonic-rs, simdutf8, rust-lexical, ryu,
  hashbrown/SwissTable, scc, papaya, crossbeam-epoch, loom, bumpalo, bytes,
  LLVM BOLT).
- **rustx-skills 1.0.0** (reviewed 2026-08-21): rule-by-rule audit of all 265
  rules in `leonardomso/rust-skills` (MIT) — 138 KEEP / 112 ADAPT / 15 MERGE,
  0 hard rejects. Full table preserved in [\_rule-matrix.md](_rule-matrix.md).
- **Microsoft Pragmatic Rust Guidelines 2026.6** (MIT, Microsoft Corporation):
  snapshot reviewed directly; adopted/adapted/rejected record in
  [\_source-adaptations.md](_source-adaptations.md).

### Empirical verification (rustc 1.98.0 compile probes, 2026-08-22)

Every toolchain claim inherited from the sources was probed against the
stable compiler rather than trusted:

| Claim                                             | Verdict                                                                  |
| ------------------------------------------------- | ------------------------------------------------------------------------ |
| `core::hint::cold_path`, `select_unpredictable`   | stable ✓                                                                 |
| `core::fmt::NumBuffer` + `<int>::format_into`     | stable ✓                                                                 |
| `f32/f64::algebraic_*`                            | stable ✓                                                                 |
| `Atomic*::from_mut/get_mut_slice/from_mut_slice`  | stable ✓                                                                 |
| strict provenance (`addr`/`with_addr`/`map_addr`) | stable ✓                                                                 |
| `str::substr_range` / `[T]::subslice_range`       | stable ✓ — **corrected: returns `core::range::Range`, not `ops::Range`** |
| `std::simd`                                       | unstable (E0658) ✓                                                       |
| `Iterator::collect_into`                          | unstable (E0658) ✓ — upstream had verified only against 1.97.1           |
| cargo `trim-paths` profile key                    | unstable — rejected by cargo 1.98 ✓                                      |

### Synthesis outcome

The three sources were compressed into `skills/`: one golden orchestrator
(SKILL.md, 25 Prime Laws), 19 numbered domain files, the one-line canon, the
blueprints, behavioral evals, and this audit archive. Conflicts were resolved
with recorded verdicts — see [\_source-adaptations.md](_source-adaptations.md).

## Revision protocol

On every MSRV bump or source refresh: re-probe the stability table above,
re-hash refreshed sources into the lock, record new adaptations with their
reasons, and never let a fetched document enter agent authority without
review (source-instruction firewall, [11-dependencies.md](11-dependencies.md)).
