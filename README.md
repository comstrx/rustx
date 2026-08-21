# ✨ RustX

<div align="center">
  <br/>
  <br/>
  <img height="280" src="https://github.com/user-attachments/assets/d1c18d6c-9f61-4782-80c1-6d7eda2dc0ad" />
  <br/>
  <br/>
</div>

[![License: AGPL-3.0](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](./LICENSE)
[![Rust 1.98+](https://img.shields.io/badge/rust-1.98%2B-orange.svg)](https://www.rust-lang.org)
[![edition 2024](https://img.shields.io/badge/edition-2024-green.svg)](https://doc.rust-lang.org/edition-guide/)
[![CI](https://github.com/comstrx/rustx/actions/workflows/ci.yaml/badge.svg?branch=main)](https://github.com/comstrx/rustx/actions/workflows/ci.yaml)
[![Release](https://img.shields.io/github/v/release/comstrx/rustx?sort=semver)](https://github.com/comstrx/rustx/releases/latest)

`rustx` is a rust workspace for building high-performance framework foundations, developer tooling, and infrastructure runtimes.

## Overview

<code>rustx</code> is the Rust workspace for a family of focused ToolX crates
maintained by [comstrx](https://github.com/comstrx). Workspace crates use the
<code>rustx-&lt;name&gt;</code> package convention and live under
<code>crates/&lt;name&gt;</code>.

## Current crates

- <code>crates/rustx</code> contains the <code>rustx</code> facade.
- <code>crates/typing</code> contains the <code>rustx-typing</code> package.

  [dependencies]
  rustx = { git = "https://github.com/comstrx/rustx", branch = "main" }

  use rustx::typing::Typing;

  assert_eq!(Typing::hello_world(), "Hello, world!");

Consumers may also depend on <code>rustx-typing</code> directly when they do not
need the facade.

## Workspace

- [rustx facade](https://github.com/comstrx/rustx/tree/main/crates/rustx)
- [rustx-typing](https://github.com/comstrx/rustx/tree/main/crates/typing)
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

<code>rustx</code> is licensed under the
[GNU Affero General Public License v3.0 only](https://github.com/comstrx/rustx/blob/main/LICENSE).
