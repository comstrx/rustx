# ✨ Rustx

<div align="center">
  <h1>Rustx</h1>
  <p>
    <strong>A production-grade Rust workspace for reusable ToolX crates.</strong>
  </p>
  <p>

<!-- prettier-ignore-start -->

[![CI](https://github.com/comstrx/rustx/actions/workflows/ci.yml/badge.svg)](https://github.com/comstrx/rustx/actions/workflows/ci.yml)
![Version](https://img.shields.io/badge/version-v0.1.0-blue.svg)
![MSRV](https://img.shields.io/badge/rustc-1.98+-ab6000.svg)
![License](https://img.shields.io/badge/license-AGPL--3.0--only-blue.svg)

<!-- prettier-ignore-end -->

  </p>
</div>

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
