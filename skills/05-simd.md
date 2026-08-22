# SIMD, SWAR, Scanning, and `memmatch`

**Load when:** building byte/text scanners, structural classification,
substring search, or CPU dispatch.

**Output:** a scalar oracle plus a routed kernel portfolio with explicit
feature, tail, and mask contracts.

## SIMD is an engine layer, not an API personality

The public API expresses semantics, never vector width or instruction set.
Production lanes on Rust 1.98:

```text
stable scalar/SWAR reference
stable core::arch / std::arch intrinsics
runtime or compile-time target dispatch
nightly-lab portable_simd experiments (std::simd is NOT stable on 1.98)
```

## The specialization portfolio

A world-class scanner is a portfolio, and the router is part of the algorithm:

```text
empty / 1-byte / tiny needle
→ scalar unrolled or word-at-a-time (SWAR)
→ candidate filter
→ vector structural scan
→ prepared multi-use finder
→ streaming/chunk-boundary engine
→ portable fallback
```

Benchmark setup separately from reuse; record crossover thresholds per
architecture. **Short-input sovereignty:** a vector engine that wins large
corpora but loses ubiquitous short inputs makes the product slower — tiny
paths avoid feature dispatch, prepared state, and wide setup unless measured.

## The dispatch ladder

1. Compile a safe baseline for the declared target floor.
2. Compile private target-specific kernels with `#[target_feature]`.
3. Detect support before selection (`is_x86_feature_detected!` et al.).
4. Cache the selected function pointer or strategy once (`OnceLock`).
5. Call the cached path without repeating detection.
6. Preserve the scalar fallback.

```rust
type FindByteFn = fn(&[u8], u8) -> Option<usize>;
static FIND_BYTE: OnceLock<FindByteFn> = OnceLock::new();

#[inline]
pub fn find_byte(haystack: &[u8], needle: u8) -> Option<usize> {
    (FIND_BYTE.get_or_init(resolve_find_byte))(haystack, needle)
}
```

Keep the resolver and kernels private so no unchecked path bypasses the CPU
invariant. Never call a `#[target_feature]` kernel without established
support. Global `-C target-feature` can emit trapping instructions on weaker
CPUs — never in a redistributable artifact; `target-cpu=native`/fleet tuning
is for controlled binaries whose deployment contract records the CPU.

## The vector contract

Each kernel declares: required CPU features and how dispatch proves them;
minimum readable bytes and tail policy; alignment assumptions (usually none
unless proven); mask meaning and lane ordering; false-positive policy and
scalar verification; UTF/text vs raw-byte semantics; whether padding/sentinels
are internal or caller-visible; input overlap and output aliasing rules.

## Kernel anatomy

```text
prologue / tiny-input path
→ wide block loop (branch-light)
→ mask extraction
→ candidate verification or state update
→ tail without out-of-bounds access
```

Load unaligned when safe and measured — alignment peeling is not automatically
faster. Convert vector comparisons into bitmasks; extract set bits with
`trailing_zeros`/clear-lowest-bit loops. Handle tails without reading outside
the allocation unless the API explicitly guarantees padded readable bytes
(see the sentinel contract in [16-patterns.md](16-patterns.md)). Never
construct references to padding or uninitialized bytes.

## SWAR before intrinsics

SIMD-within-a-register processes multiple bytes with ordinary integer ops:
portable, stable, low setup for small inputs, well optimized by LLVM. Use for
digit classification, ASCII detection, delimiter masks, zero-byte detection,
short fixed-width parsing — when it benchmarks well.

## Mask algebra before branches

For classification-heavy parsers, design the mask algebra first:

```text
quote_mask · backslash_mask · structural_mask · whitespace_mask · utf8_error_mask
```

Bitwise operations transform these into "outside string," "escaped,"
"candidate delimiter" positions without a branch per byte. Keep semantic state
(string/escape parity) separate from physical lane width so scalar and SIMD
paths share one model. This is the architectural heart of high-throughput
parsers: vectorize classification, then consume sparse structural positions.

## Branch strategy

- Predictable branch → keep it; it may beat branchless work.
- Unpredictable cheap selection → evaluate `core::hint::select_unpredictable`
  (stable on 1.98) or mask selection.
- Rare branch → outline it; optionally `core::hint::cold_path()` (stable).
- Expensive arms → avoid branchless evaluation that computes both.
- Parser state per byte → block classification or table/mask transitions.

## Prefetch, unroll, pipeline

Software prefetch only for large predictable traversals with independent work
— it often hurts; never by folklore. Unroll only to hide latency with
independent accumulators, reduce loop control, or match vector widths; stop
when spills or instruction-cache misses erase the gain.

## `memmatch` — searching is a strategy portfolio

```text
needle 0        → semantic edge case
needle 1        → memchr-style byte scan
needle 2–3      → multi-byte candidate scan
short fixed     → vector/SWAR first+last-byte filter + verify
medium          → vector prefilter + robust exact algorithm
long/repeated   → preprocessed finder; Two-Way or chosen engine
many needles    → automaton/filter for set search
```

Thresholds are benchmark data, not constants of nature.

**Prepared finder:** repeated search amortizes CPU dispatch, needle
fingerprinting, factorization/skip tables, selected candidate bytes:

```rust
let finder = MemMatch::prepare(needle);
for chunk in chunks {
    finder.find(chunk);
}
```

**Candidate filtering:** scan discriminating positions (first/last or rare
bytes) in parallel, verify only candidates; always keep a worst-case-safe
exact fallback — heuristics must not compromise pathological complexity.

**Streaming search:** preserve overlap state to catch needles crossing chunk
boundaries; define offset semantics, empty-needle semantics, maximum retained
prefix, fragmented input behavior, reset behavior. Never concatenate chunks
merely to reuse a contiguous API.

**Bytes vs text:** the search engine operates on bytes. Unicode semantics
(normalization, graphemes, case folding) belong to a higher text layer. Byte
substring search is not Unicode semantic search.

## Numerical vectorization modes

Rust 1.98's stable algebraic float ops (`algebraic_add` …) permit
reassociation and vectorization by changing numerical semantics. Expose two
explicit modes — `Strict` and `Algebraic` — resolve the mode outside the inner
loop into a specialized plan, and never silently substitute algebraic ops in a
general `Num` API.
