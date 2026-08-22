# Unified Error Engineering

**Load when:** changing error representation, conversion, diagnostics, panic
policy, or failure telemetry.

**Output:** a stable programmatic error contract with cold rendering and
explicit panic containment.

## Errors are data first, prose last

Every RustX crate speaks the unified error vocabulary — structured, compact,
composable, cheap on success paths:

```text
ErrorCode      stable machine-readable identity  (NonZeroU32, repr(transparent))
ErrorKind      domain category for broad matching
ErrorDetail    compact typed payload / offsets / static data
Error          owned cross-crate envelope { code, kind, flags, context }
ErrorRef<'a>   borrowed/internal view where useful
Report         rich display/context/backtrace boundary
```

Capability tiers — lower tiers never depend on higher rendering machinery:

```text
core identity       code + kind + compact location/offset
library context     typed fields / source classification
application render  localized message + chain + backtrace + telemetry
```

The engineering behind `thiserror`/`anyhow` (typed library errors, ergonomic
dynamic reporting at the outermost edge, context, `?` conversion, preserved
chains) is provided by `core::error` — external error crates are never the
cross-workspace semantic contract.

## Error invariants

- Every public fallible API returns a RustX-compatible error or a documented
  adapter; external errors convert at crate boundaries via `From` (canonical
  conversion — not scattered `map_err`).
- Machine-readable code never depends on formatted text.
- Formatting is lazy; backtrace capture is policy-driven, never automatic in
  low-level failures.
- Source chains are preserved (`std::error::Error::source` interop) without
  forcing inner layers to allocate; never flatten a useful chain into a string.
- Context adds semantics (operation, entity, offset, recovery class) — not
  duplicate prose at every layer.
- Positions are offsets/ranges; line/column/snippets derive lazily.
- The success path never allocates to prepare for a possible error.
- Messages: lowercase, no trailing punctuation; document failure modes under
  `# Errors`.

## Hot/cold split

```rust
#[inline]
fn parse_port(bytes: &[u8]) -> Result<u16, Error> {
    match parse_port_fast(bytes) {
        Some(port) => Ok(port),
        None => parse_port_error(bytes),
    }
}

#[cold]
#[inline(never)]
fn parse_port_error(bytes: &[u8]) -> Result<u16, Error> {
    core::hint::cold_path();
    Err(Error::invalid_port(bytes))
}
```

Never build strings, snippets, or dynamic chains before failure is known.
Inspect closure codegen (`ok_or_else`) in truly tiny kernels — an explicit
`match` may lay out better.

## Panic policy

Libraries return `Result` for recoverable, environmental, and input failures.
Panics mean "stop: a programming bug or violated internal invariant" — and the
workspace lint wall additionally denies `panic!`/`unwrap`/`expect`/indexing in
library code, so invariants surface through `assert!`/`debug_assert!` with
messages that name the violated invariant and key values (no secrets), or
through structured errors.

- Detected programming bugs are panics/assertions, never `Err` — but reach for
  a type that makes the state unrepresentable first.
- `catch_unwind` is valid only at a containment boundary that can discard or
  restart potentially damaged state; a caught payload is not repaired
  invariants.
- Unchecked hot variants exist only when a checked counterpart exists, the
  precondition is precise, the value is measured, and the unsafe contract is
  explicit.
- Applications may choose `panic = "abort"` as deployment policy; libraries
  remain sound under unwinding regardless (the workspace `release` profile
  aborts, `dev`/tests unwind — code must be correct under both).

## Error code stability

Codes are part of the intellectual ABI: namespaced by domain, non-zero,
compact, stable across text changes, documented with retry/recovery
classification, extensible without renumbering. Resource-limit failures use a
stable resource-limit code family.

## Failure telemetry

Log an error exactly once, with its full source chain, at the boundary that
handles it. Never log secrets or PII. See [14-security.md](14-security.md)
for the telemetry cost contract.
