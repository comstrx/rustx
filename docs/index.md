# RustX Documentation

`rustx` is a Rust workspace of focused, composable crates meant to sit at the
bottom of everything else — a foundation, not an application.

## Workspace layout

Published crates live under `crates/<name>` and are packaged as `rustx-<name>`,
with `crates/rustx` as the facade that re-exports them all. Four support crates
sit at the root and are never published:

| Path       | Purpose                                                     |
| ---------- | ----------------------------------------------------------- |
| `tests`    | Public-API contract tests, linked the way a consumer links. |
| `benches`  | Criterion benchmarks.                                       |
| `examples` | Runnable examples.                                          |
| `bloats`   | Binary-size probes for `cargo bloat`.                       |

The workspace manifest is the single source of truth for which crates exist;
`bash scripts/run.sh meta` prints the current set.

Keeping test, bench, example and size-probe code out of the published crates
buys two things: the published crates carry no dev-dependencies, and the
contract tests can only reach the public API — API-first is enforced by the
layout, not by discipline.

## Crate anatomy

Every crate follows the same shape, so moving between them costs nothing:

```text
crates/<name>/
  Cargo.toml    inherits every workspace field, opts into the lint policy
  LICENSE-MIT   the license text travels with the package
  LICENSE-APACHE
  src/lib.rs        crate docs, module declarations, public re-exports
  src/prelude.rs    the crate's vocabulary, glob-importable
  src/<concept>.rs  the implementation, one file per concept
```

The anatomy is not a convention anyone has to remember. `bash scripts/run.sh new
<name>` generates it and registers the crate in the workspace members, the
workspace dependencies, and both facade re-exports; `bash scripts/run.sh conform`
then verifies every crate against the same contract and fails the build on any
drift. `conform` runs inside `ci-lint`, so a crate cannot enter the tree without
matching.

## Usage

Take the whole foundation through the facade:

```toml
[dependencies]
rustx = { git = "https://github.com/comstrx/rustx", branch = "main" }
```

```rust
use rustx::prelude::*;
```

Every member crate is reachable as `rustx::<name>`, and `rustx::prelude` is the
union of the member preludes. Depending on a single crate directly works too —
the paths are the only difference.

Nothing is on crates.io yet. Publication waits for a first release that has been
designed, benchmarked and tested as a whole.

## Gates

`scripts/run.sh` drives every check. During development the loop is
`write → self-review → check`; the formal gates run when they are asked for.

```bash
bash scripts/run.sh check          # compile checks, all crates and targets
bash scripts/run.sh test           # full test suite
bash scripts/run.sh clippy         # lint gate for publishable crates
bash scripts/run.sh clippy-strict  # lint gate for the whole workspace
bash scripts/run.sh ci-lint        # conform + taplo + prettier + typos
bash scripts/run.sh audit-check    # advisories, bans, licenses, sources
bash scripts/run.sh coverage       # cargo-llvm-cov
bash scripts/run.sh miri           # UB detection under the interpreter
bash scripts/run.sh sanitizer      # asan/tsan/msan/lsan
bash scripts/run.sh doctor         # environment diagnostics
```

Run `bash scripts/run.sh --help` for the full command list.

`sanitizer` rebuilds `std` with instrumentation by default, which needs a
toolchain whose `rust-src` can resolve its own lockfile. Pass `--no-build-std`
to instrument workspace code only.

`semver` is release-time only: it needs a published baseline to diff against,
so it runs on `refs/tags/v*` and on manual dispatch, not on every push.

### Platforms

The gates are expected to behave identically on Linux, macOS and Windows via
Git Bash or MSYS2, and under WSL. `scripts/core/bash.sh` runs before anything
else and re-execs into bash 5 or newer, so the rest of the tree can rely on
namerefs, associative arrays and `mapfile`. Where a tool differs between GNU and
BSD userland the script picks a working implementation; where a platform genuinely
cannot run a gate, it fails with the reason and the fix rather than skipping.

CI exercises `scripts/run.sh` on `ubuntu-latest`, `macos-latest` and
`windows-latest` for the stable, nightly and MSRV profiles.

`scripts/` is scoped to this repository. Modules live flat under
`scripts/module/`, one file per command group plus `base.sh` for the cargo
plumbing they share, with `scripts/core/` as the generic shell library and
`scripts/initial/` as the loader. Every function in the tree is reachable from a
command — there is no unused surface to maintain.

## Profiles

| Profile           | Use                                                                                      |
| ----------------- | ---------------------------------------------------------------------------------------- |
| `release`         | Shipping. Maximum optimization, symbols stripped, no runtime checks.                     |
| `release-checked` | `release` with debug assertions and overflow checks, for fuzz and sanitizer runs.        |
| `profiling`       | `release` with full debug info and no stripping, so `samply` and `flame` resolve frames. |
| `bench`           | `release` with line tables, so benchmarks stay profilable.                               |

## Lint policy

The workspace denies `clippy::{all, cargo, nursery, pedantic}` plus a curated
restriction set, and treats warnings as errors. Two rules make the policy hold:

- Every crate opts in with `[lints] workspace = true`, including the
  non-published ones. A crate outside the policy is a hole in it.
- `clippy::allow_attributes` and `clippy::allow_attributes_without_reason` are
  denied, so an escape hatch must be `#[expect(..., reason = "...")]` — scoped,
  justified, and failing the build once it stops being needed.

See `Cargo.toml` for the lint set, `.clippy.toml` for its thresholds and
disallowed items, and `.deny.toml` for the supply-chain policy.

### Unsafe

`unsafe_code` is denied workspace-wide. `unsafe` is still available — it enters
through a scoped expectation on the smallest item that needs it:

```rust
#[expect(unsafe_code, reason = "SIMD load requires a raw pointer; see the safety note")]
#[inline]
fn head(bytes: &[u8]) -> Option<u8> {
    if bytes.is_empty() {
        return None;
    }

    // SAFETY: the emptiness check above guarantees index 0 is in bounds.
    Some(unsafe { *bytes.get_unchecked(0) })
}
```

Three properties make this a real gate rather than a formality. `expect`
self-expires: delete the `unsafe` and the attribute becomes an unfulfilled
expectation that fails the build, so an exemption cannot outlive its reason.
`clippy::undocumented_unsafe_blocks` requires the `// SAFETY:` note, and
`clippy::multiple_unsafe_ops_per_block` keeps one unsafe operation per block so
each note describes exactly one thing. The pointer and provenance lints in
`Cargo.toml` cover the failure modes that only appear once raw pointers exist,
and Miri plus the sanitizers exercise them.
