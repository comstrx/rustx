# ✨ RustX

<div align="center">
  <br/>
  <img height="230" src="https://github.com/user-attachments/assets/d1c18d6c-9f61-4782-80c1-6d7eda2dc0ad" />
  <br/>
  <br/>
</div>

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Rust 1.98+](https://img.shields.io/badge/rust-1.98%2B-orange.svg)](https://www.rust-lang.org)
[![edition 2024](https://img.shields.io/badge/edition-2024-green.svg)](https://doc.rust-lang.org/edition-guide/)
[![CI](https://github.com/comstrx/rustx/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/comstrx/rustx/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/comstrx/rustx?sort=semver)](https://github.com/comstrx/rustx/releases/latest)

`rustx` is a production-grade Rust workspace of reusable, composable crates.

## Overview

<code>rustx</code> is the Rust workspace for a family of focused ToolX crates
maintained by [comstrx](https://github.com/comstrx). Workspace crates use the
<code>rustx-&lt;name&gt;</code> package convention and live under
<code>crates/&lt;name&gt;</code>.

## Usage

Take the whole foundation through the facade:

```toml
[dependencies]
rustx = { git = "https://github.com/comstrx/rustx", branch = "main" }
```

```rust
use rustx::prelude::*;
```

Every member crate is reachable as <code>rustx::&lt;name&gt;</code>, and
<code>rustx::prelude</code> is the union of the member preludes. Consumers who
need exactly one crate may depend on it directly instead; the paths are the only
difference.

Run <code>bash scripts/run.sh doc-open</code> for the current crate list and the
full API — the workspace manifest is the single source of truth for what ships.

## Workspace

- [Crates](https://github.com/comstrx/rustx/tree/main/crates)
- [Documentation](https://github.com/comstrx/rustx/tree/main/docs)

## Development

    bash scripts/run.sh check
    bash scripts/run.sh test
    bash scripts/run.sh ci-lint

The minimum supported Rust version is <code>1.98.0</code>.

## Community

- [Issues](https://github.com/comstrx/rustx/issues)
- [Discussions](https://github.com/comstrx/rustx/discussions)
- [Contributing](https://github.com/comstrx/rustx/blob/main/CONTRIBUTING.md)
- [Security](https://github.com/comstrx/rustx/blob/main/SECURITY.md)
- [Support](https://github.com/comstrx/rustx/blob/main/SUPPORT.md)

## License

<code>rustx</code> is dual-licensed under either
[MIT](https://github.com/comstrx/rustx/blob/main/LICENSE-MIT) or
[Apache-2.0](https://github.com/comstrx/rustx/blob/main/LICENSE-APACHE), at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this work by you, as defined in the Apache-2.0 license, shall be
dual-licensed as above, without any additional terms or conditions.
