# Security, FFI, and Observability

**Load when:** inputs are hostile, crossing ABI/system boundaries, or adding
telemetry/resource controls.

**Output:** a bounded threat/ABI contract whose telemetry and failure behavior
preserve hot-path performance.

## Threat-model the hot path

For any public parser, map, cache, decompressor, protocol, or allocator,
define: attacker-controlled bytes/keys/lengths; CPU amplification; memory
amplification; collision/worst-case algorithm behavior; stack-depth risks;
integer overflow and truncation boundaries; retained-memory and reclamation
risks; timing/side-channel requirements; cancellation and cleanup. Security
is a performance dimension: a fast table that collapses under adversarial
collisions is not fast.

## Resource budgets are first-class

Compile budgets into an immutable plan:

```text
max_input · max_output · max_depth · max_items · max_token
max_allocation · max_retired_memory · max_inflight · max_queue · max_work_per_poll
```

Failure uses a stable resource-limit error code. No "best effort" unbounded
behavior. The adversarial complexity dossier records maxima for: bytes,
elements, nesting, key/string/number lengths, allocations and retained
memory, collision exposure, backtracking/rescans, queue depth and retry
fan-out, CPU per unit input, diagnostic output size.

## Algorithmic-complexity defenses

Seeded HashDoS-resistant hashing for untrusted keys; robust exact fallbacks
behind heuristic filters; depth limits or iterative parsers instead of
unbounded recursion; bounded automata semantics; admission/eviction that
prevents cache pollution; batch fairness so one oversized job cannot
monopolize a worker.

## Secret data

Never implement cryptographic primitives or ad-hoc constant-time code to save
a dependency. For sensitive buffers: approved zeroization whose writes cannot
be optimized away; minimal copies and lifetime; no logs/errors/core dumps;
defined pool-reuse clearing; remember `String`, formatting, and shared
ownership duplicate data. **Memory-safe does not mean secret-safe.**

## Validation proof types

When repeated validation is expensive, carry the proof in a type:

```text
&[u8] --validate--> Utf8Bytes<'a> --parse--> JsonRaw<'a> --query--> JsonRef<'a>
```

Safe APIs trust the invariant carried by the type; raw unchecked constructors
remain unsafe or crate-private.

## Failure containment

Corrupt input never mutates shared state before validation completes; publish
new state atomically after full construction; bound retries with backoff;
make loaders idempotent or identity-keyed; one malformed request never
poisons process-global state.

## FFI translation architecture

Business semantics live in ordinary safe crates. The FFI crate owns only:
ABI-safe declarations and version negotiation; handle validation and
ownership translation; buffer length/capacity contracts; error-code
conversion; panic containment (panics never cross a non-unwind ABI);
allocation/deallocation pairing; platform loading. Follow established
`-sys`/wrapper naming conventions.

Never move Rust-owned `String`/`Vec`, trait objects, references, `TypeId`,
unwinding, or allocator assumptions across an ABI — even Rust-to-Rust. Opaque
handles and C-compatible records; allocate and free on the same side unless
an allocator ABI is explicitly contracted. `repr(C)`/`repr(transparent)` only
where needed; `repr(Rust)` never crosses. Validate lengths, nullability,
alignment, ownership on entry; define string encoding and termination; map
foreign errors into the core error vocabulary at the boundary.

**Dynamic-library isolation:** each dylib may carry a different compiler,
dependency graph, and allocator. Only portable bytes/scalars/declared layouts
cross; correctness never depends on shared statics or identical
monomorphizations.

## Mockable system boundary

Clocks, entropy, filesystem, network, process, and environment access are
injectable at the boundary when correctness tests need control — a narrow
concrete service/context interface with a deterministic test implementation.
Never trait-abstract every syscall in the hot path.

## Observability without hot-path pollution

Logging/tracing is not part of the lowest memory/type substrate. Engines emit
structured events through a minimal optional sink or counters; formatting,
exporters, rotation, and telemetry backends live above. Libraries emit —
applications install subscribers/sinks and own level policy.

Rules: disabled observability compiles to nearly nothing; no allocation or
formatting on successful hot operations; structured key-value fields, not
interpolated strings; no unbounded label cardinality; redact secrets before
event construction; aggregate counters per worker and merge; sample expensive
traces; stable error codes and operation IDs for correlation; log each error
exactly once with its full chain; benchmark with production observability on
as well as off.

Useful optional engine counters: allocations/growths, scalar-vs-SIMD path
selections, small-vs-large path selections, hash probes/collisions, cache
hits/evictions, arena chunks/peak bytes, escaped-string materializations,
queue saturation/retries. Counters must not become the bottleneck.
