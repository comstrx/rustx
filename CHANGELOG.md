# Changelog

Every release of RustX.

## v0.1.1 — 2026-10-17

- `http::Router::check` names the first route pattern that is malformed or conflicts with an earlier one, and a server refuses to start on it (`bad_route`, a `Config` failure) instead of answering 404 for a route that was declared.

## v0.0.2 — 2026-10-17

- One facade over log, error, memory, parse, validate, thread, rand, crypto, file, net, http and process, with capability features.
- Engine-owned types on every surface and every failure declared once in a fault contract.
- `process::Args::version` lets every binary answer `--version` with `<name> <version>`.
- Tests, benches, fuzz, miri and semver gates on Linux, macOS and Windows.
