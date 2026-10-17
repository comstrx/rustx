# Changelog

Every release of RustX.

## v0.0.2 — 2026-10-17

- One facade over log, error, memory, parse, validate, thread, rand, crypto, file, net, http and process, with capability features.
- Engine-owned types on every surface and every failure declared once in a fault contract.
- `process::Args::version` lets every binary answer `--version` with `<name> <version>`.
- Tests, benches, fuzz, miri and semver gates on Linux, macOS and Windows.
