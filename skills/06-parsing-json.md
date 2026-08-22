# JSON and Parser Engine Architecture

**Load when:** implementing or changing JSON parse, query, DOM, typed decode,
streaming, or serializer paths — the doctrine generalizes to every parser.

**Output:** an ownership-explicit product route with validation, security
limits, and a differential oracle.

## `Json` is multiple products behind one vocabulary

| Product                   | Best default                                  |
| ------------------------- | --------------------------------------------- |
| known schema, one pass    | direct typed decode                           |
| many random queries       | structural index/tape or compact DOM          |
| few paths from huge input | on-demand cursor/path extraction              |
| unbounded stream          | incremental event/record iterator             |
| mutable general value     | owned DOM with explicit allocation profile    |
| borrowed inspection       | input-backed `JsonRef<'a>` with lazy unescape |

```text
JsonRef<'a>    borrowed validated/lazy value
JsonOwned      independent owned value
JsonTape<'a>   structural/indexed document for repeated traversal
JsonArena      arena-backed document
JsonRaw<'a>    validated raw slice, deferred decoding
JsonNumber<'a> raw or parsed number with explicit precision policy
```

The DSL makes transitions easy; ownership and materialization stay observable
in types. Routing is explicit or profile-driven — never hidden semantics that
make latency unpredictable.

## Two-stage architecture for large inputs

**Stage 1 — classify and validate (vectorizable):** fuse compatible work —
structural character discovery, quote/backslash state, whitespace
classification, UTF-8 validation, optional newline indexing — into masks or a
compact structural index. Avoid one branch per byte. Track UTF-8, escapes,
number grammar, depth, and balance as separate logical invariants even when
one physical scan advances several (fusion must not defeat auditability).

**Stage 2 — consume structure:** a tight state machine over structural
positions builds tape entries or typed values, validates grammar and nesting,
parses numbers, materializes/unescapes only when required, enforces limits,
attaches offset-precise errors.

**Small-input path:** a direct scalar parser beats SIMD setup below a
benchmark-derived threshold. Maintain the tiny route.

## Strings: borrow first, unescape on demand

A JSON string has three states:

```text
raw token bytes → validated escaped view → materialized decoded text
```

```text
no escapes → JsonStr::Borrowed(range)
escapes    → unescape into arena/owned buffer
```

Never allocate every key and string during parse. Cache materialization only
when reuse justifies retained memory. In-place unescaping of a uniquely owned
buffer may exist as an explicit destructive fast path whose API makes input
invalidation undeniable.

## Objects are not automatically hash maps

Building a hash table for every object pays hashing, allocation, metadata,
and random writes even when iterated once. Route by size and usage:

```text
tiny object            → contiguous pairs + linear/length-assisted match
medium immutable       → sorted or indexed compact entries
large / hot queries    → lazily built hash index
schema-known           → generated/direct field dispatch, no generic map
```

Duplicate-key and ordering semantics are contract choices (reject / first /
last / preserve) — every path agrees, and a lazy index never silently erases
duplicate information.

## Key matching for typed decode

Compare length first; use packed/SWAR comparison for short keys; generated
perfect-hash or decision trees for static schemas when worthwhile; skip
unknown values without materializing them. Generated dispatch code size is
measured, not assumed.

## Numbers

Separate syntax, precision, and target conversion:

```text
raw validated span
→ integer fast path | exact decimal/raw | f64 conversion | arbitrary-precision adapter
```

Parse multiple digits per iteration (SWAR/SIMD) where measured; detect
overflow before committing a narrow integer; preserve raw numbers for
lossless workflows; use proven fast float algorithms with a correct fallback;
never silently lose precision in a lossless `Json` API. For integer output,
`<int>::format_into(&mut NumBuffer)` (stable 1.98) avoids general formatting
machinery.

## Serialization

Write directly into a caller/reusable `Buffer`; reserve from size hints;
SIMD/SWAR-scan strings for escapable bytes and bulk-copy clean spans; format
integers into fixed buffers; use a correct fast float formatter; no virtual
dispatch per scalar; pretty-printing separate from the compact hot path;
expose `write_to`/`encode_into` rather than forcing `String` allocation.

## Input safety limits are performance features

Untrusted parsing supports bounded policies: max bytes, nesting depth, string
length, number token length, member/element counts, total materialized
output, arena growth, duplicate-key policy, cooperative budget. Failure is
bounded with a stable resource-limit error code. See
[14-security.md](14-security.md).

## Error location without hot-path formatting

Track compact offsets during parsing; derive line/column/snippet lazily at
display. Maintain a newline index only when requested or amortized — never
count lines byte-by-byte per successful token, and never let location
reporting rescan the whole prefix accidentally.

## Serde boundary

Serde compatibility is an adapter behind the `serde-interop` feature. It never
dictates the internal DOM, parser phases, error representation, or ownership
model. When enabled, apply the interop conventions (rename_all, default,
skip_serializing_if, deliberate enum tagging, deny_unknown_fields where the
contract wants it, try_from validation) at the boundary only. RustX is
ecosystem-compatible without being ecosystem-shaped.
