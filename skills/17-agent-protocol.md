# Agent Execution and Delivery

**Load when:** implementing or refactoring code, coordinating rounds, running
requested gates, or declaring completion.

**Output:** a compiling change with truthful proof status and no context or
gate waste.

## Boot sequence for every task

1. Read `skills/SKILL.md`; load only the routed files whose content can
   change a decision — context locality is the agent equivalent of cache
   locality.
2. Read the target crate's `Cargo.toml`, `lib.rs`, facade, error, and
   adjacent engine modules; find existing RustX vocabulary and ADRs.
3. Identify dependency direction and the public semver surface.
4. State the semantic contract and workload contract.
5. List ownership, allocation, concurrency, and unsafe invariants.
6. Search for an existing RustX primitive before adding any abstraction or
   dependency.

Never start by generating files from generic Rust templates. Show the design
in code and artifacts — do not sermonize about compliance.

## Implementation sequence

```text
A. Define behavior and cost contract.
B. Select representation and ownership.
C. Implement the simplest correct scalar/safe reference.
D. Compile (bash scripts/run.sh check).
E. Add prepared state, reuse, or data-layout improvement.
F. Compile.
G. Add specialized kernels/unsafe only where justified.
H. Compile.
I. Differentially self-review specialized semantics against the reference.
J. Stop without formal gates unless the user requests them.
```

Compilation is mandatory after every coherent round and is not a gate. A
targeted test/benchmark explicitly requested is fine — On-Demand is about
control, not omission. Never silently convert an implementation task into a
gate-running task.

## Mandatory self-review after each round

**Architecture** — crate boundaries and dependency direction preserved? A
duplicate primitive created? Facade independent from backend? External type
leaked into public semantics?

**Ownership** — who owns every allocation? Can borrowed output outlive input,
guard, arena, or buffer? Did convenience introduce a hidden clone/`Arc`/
allocation? Can consuming APIs reuse storage?

**Memory** — allocations/copies in common and worst paths? Capacity
boundaries and OOM? Partial initialization panic-safe? ZST, alignment,
overflow, huge lengths handled?

**Safety** — every unsafe precondition explicit and established? A reference
created too early? Provenance preserved? Aliasing or concurrent access able
to violate invariants? `Send`/`Sync` correct for all reachable state?

**Performance** — work removed before cleverness added? Small-input path
protected? Setup amortized? Hot code polluted by diagnostics? Generics
expanding code size?

**Failure** — parse error, panic, cancellation, closed channel, poisoned
state? Error construction cold and structured? Partially committed state
exposed?

**Portability** — stable baseline compiles without non-baseline CPU
assumptions? Every target-feature call guarded? Accidental nightly?

**DSL** — allocation/ownership transitions visible in names? Common path
short and discoverable? Easy API delegating to the same fast engine?

## Refactor protocol

Preserve semantics before changing structure; small coherent
transformations; compile each round; never replace whole files for a narrow
concern; no formatting churn; do not generalize until two real consumers
share the invariant. A refactor is valid only if it improves at least one
dimension — semantics/safety/duplication-of-policy/dependencies/performance/
DSL/compile surface — without silently regressing a higher-priority one.
Large line churn without a semantic delta is suspect, especially in
agent-generated work.

## Performance and unsafe protocols

An optimization may be proposed from architecture knowledge, never claimed
without measurement. State the hypothesis first (see
[02-workloads.md](02-workloads.md)). Before unsafe: attempt a safe
representation; inspect whether the compiler already emits equivalent code;
reduce to the smallest capsule; write the safety case with the code; preserve
the reference implementation; prepare differential/Miri/fuzz assets for the
next requested checkpoint (see [04-unsafe.md](04-unsafe.md)).

## Dependency protocol

Never add a dependency because it is familiar. Name the capability → check
workspace-approved dependencies → check whether `core` owns it → evaluate the
admission dossier → hide it behind a RustX boundary → use workspace
version/features (see [11-dependencies.md](11-dependencies.md)).

## Gate checkpoint protocol

When gates are explicitly requested, select all relevant configured checks —
format, compile matrix, clippy, tests, coverage, property/differential,
Miri, fuzz corpus, model checking, audit/deny/vet, udeps, semver, MSRV,
benchmarks — via `bash scripts/run.sh <gate>`, and report exact failures
without hand-waving. Never weaken code or suppress meaningful diagnostics to
make a dashboard green. Green CI proves the exercised paths, not semantic
correctness — coverage measures execution, not intent.

## Completion honesty

Use exact language — never transform "not run" into "should pass":

- **compiled:** command and scope;
- **self-reviewed:** invariants/skills checked;
- **tested / benchmarked / fuzzed:** only when actually run;
- **hypothesis:** expected benefit not yet gated;
- **blocked:** concrete missing prerequisite;
- **deferred:** explicit non-critical obligation.

## The final delivery test

Before declaring a component complete, answer precisely — hand-waving on any
question means the component is not architecturally finished:

1. What exact semantics does the public API guarantee?
2. Which ownership transitions allocate, clone, share, or consume?
3. What is the common-case physical representation?
4. What are the small, large, adversarial, and fallback paths?
5. Which setup costs are amortized, and where?
6. How many allocations, passes, hashes, and copies occur?
7. Where is unsafe code, and what is its complete safety case?
8. How are panic, cancellation, and partial initialization handled?
9. How are CPU features selected without violating the artifact's
   compatibility contract?
10. How are concurrency and reclamation proven?
11. Why does the DSL stay easy without hiding material cost?
12. Which dependencies are used, and how are they contained?
13. What scalar/reference implementation defines truth?
14. What proof assets exist for the next requested gate checkpoint?
15. Did the code compile after the final round?
16. What evidence level supports every performance claim?

## Skill self-improvement loop

When a real task exposes a repeated mistake: capture the exact failure mode;
add the smallest rule or template that prevents it, in the narrowest relevant
skill file; verify the change against the live toolchain; remove redundant
prose that does not alter behavior. The skills must buy correctness per token
— more doctrine is not more intelligence.
