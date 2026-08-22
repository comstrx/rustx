# AGENTS.md

## What this repository is

`rustx` is a production-grade Rust workspace of reusable, composable crates —
a foundation meant to sit at the bottom of everything else. Published crates
live under `crates/<name>` as `rustx-<name>`, re-exported through the
`crates/rustx` facade. One workspace
version, edition 2024, stable Rust 1.98, `no_std`-first. Support crates
(`tests/`, `benches/`, `examples/`, `bloats/`) live outside the published
crates. `docs/index.md` describes the layout; `bash scripts/run.sh` is the
only tool runner.

## The sacred base: `skills/`

**[`skills/`](skills/) is the mandatory engineering constitution for building
this tool.** Every design, implementation, review, optimization, and claim in
this repository answers to it. It is not advisory.

Start at [`skills/SKILL.md`](skills/SKILL.md) — the golden skill: authority
order, the Prime Laws, the decision order, and a routing table into the
numbered domain files. Load only the routed files the task needs.

## Non-negotiables (details live in the skills)

- **Compile continuously, gate on demand.** `bash scripts/run.sh check` after
  every coherent round. Formal gates (fmt, clippy, tests, miri, audit, bench,
  …) run only when explicitly requested.
- **The lint wall in the root `Cargo.toml` and `.clippy.toml` is the law.**
  Never bypass it; the only escape is a narrow `#[expect(..., reason)]`.
  Unsafe is sanctioned — under the full proof discipline of
  [`skills/04-unsafe.md`](skills/04-unsafe.md).
- **Claims require evidence.** No "fastest/zero-cost/allocation-free" without
  the artifacts defined in [`skills/13-proof.md`](skills/13-proof.md).
- **Guardrail files** — `.github/**`, `scripts/**`, root `Cargo.toml`, policy
  dotfiles, `supply-chain/**` — are never edited to make a change pass.
- **Git is operator-driven.** Agents do not stage, commit, tag, or push.

## Definition of done

The change compiles, respects the skills it touched, reports honestly what
was verified and what remains unproven, and passes the 16-question delivery
test in [`skills/17-agent-protocol.md`](skills/17-agent-protocol.md).
