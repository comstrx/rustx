# ✨ RustX

<div align="center">
  <br/>
  <img height="230" src="https://github.com/user-attachments/assets/d1c18d6c-9f61-4782-80c1-6d7eda2dc0ad" />
  <br/>
  <br/>
</div>

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](#license)
[![Rust 1.98+](https://img.shields.io/badge/rust-1.98%2B-orange.svg)](https://www.rust-lang.org)
[![edition 2024](https://img.shields.io/badge/edition-2024-green.svg)](https://doc.rust-lang.org/edition-guide/)
[![CI](https://github.com/comstrx/rustx/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/comstrx/rustx/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/comstrx/rustx?sort=semver)](https://github.com/comstrx/rustx/releases/latest)

`rustx` is a production-grade Rust workspace of reusable, composable crates.

## Overview

<code>rustx</code> is the Rust workspace for a family of focused ToolX crates
maintained by [comstrx](https://github.com/comstrx). Member crates use the
<code>rustx-&lt;name&gt;</code> package convention and live under
<code>crates/&lt;name&gt;</code>, with <code>crates/rustx</code> as the facade
that re-exports them all.

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

Copyright © 2026 Abdulrahman Yasser (comstrx).

Licensed under the [Apache License, Version 2.0](./LICENSE).
