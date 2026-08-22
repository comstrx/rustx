# Patterns, Decision Matrices, and Red Flags

**Load when:** choosing among plausible designs, reviewing a refactor, or
diagnosing architectural smell.

**Output:** a named pattern/profile selected by workload, plus the red flags
explicitly ruled out.

## High-leverage patterns

The recurring mechanisms behind elite engines — selected by workload, never
mandatory everywhere:

- **Parse once, represent compactly, query many** — structural tape/index
  amortized over repeated traversal; offsets/tags compact, source borrowed.
- **Direct decode when the destination is known** — never build a generic
  graph just to destroy it.
- **Filter broadly, verify narrowly** — SIMD/SWAR locates candidates; exact
  verification runs only at candidate positions.
- **Separate metadata from payload** — scan compact control bytes/fingerprints
  densely; touch payloads on match (SwissTable's essence).
- **Sparse structural stream** — transform dense bytes into positions/masks
  once; drive a low-branch state machine over interesting locations.
- **Build mutable, freeze immutable** — phase-oriented ownership removes
  synchronization and simplifies read-phase lifetimes.
- **Local-first aggregation** — per-worker counters/buffers/queues, merged in
  batches; replaces cache-line ping-pong with sequential local work.
- **Lazy indexing/materialization** — cheapest representation first; hash
  index, decoded string, line map built on demand.
- **Sentinel and padding contracts** — internal buffers may guarantee readable
  padding for vector tails only when allocation guarantees the bytes, they are
  initialized, no reference claims beyond semantic length, and the contract is
  private and tested. Never overread arbitrary caller slices.
- **Stable handles** — typed indices/generations beat reference-heavy mutable
  graphs: compact storage, movement, serialization, stale detection.
- **Two-level APIs** — a simple facade for the 90% case delegating to the same
  expert engine; never a slow "easy implementation" beside a fast one.
- **Normalize at the boundary** — ergonomic inputs at the edge, one conversion
  to a narrow internal view; conversion traits never propagate into hot
  internals.
- **Transactional mutation** — construct separately, swap/publish once: panic
  safety, reader consistency, better cache behavior.
- **Copy to become contiguous** — one bounded copy can beat fragmented
  zero-copy by unlocking locality and SIMD.
- **Static tables and generated dispatch** — compile-time tables, perfect
  hashes, decision trees for known schemas; code size measured.
- **Batch across abstraction boundaries** — batch parse/lookup/insert/encode/
  I/O amortizes virtual calls, locks, syscalls, detection, wakeups.
- **Compress the common case** — dominant state in fewest bytes/branches;
  rare complexity spilled to side storage (inline short strings, compact
  error + boxed cold context, small-object pair array + lazy index).
- **Async prelude extraction** — heavy synchronous preparation before the
  future; temporaries die before the first `.await`.
- **Capability quarantine** — strategic dependency behind a small adapter
  exporting RustX types and errors; replacement power preserved.
- **Error tokenization** — compact codes/offsets through hot layers; rich
  rendering at the outer boundary.
- **Proof-carrying threshold** — every magic number paired with its workload
  and benchmark provenance; fallback correctness independent of it.
- **Correctness-preserving specialization** — specialized kernels generated
  from one semantic source or tested differentially; specialization without a
  shared oracle becomes semantic drift.

## Decision matrices

**Ownership:**

| Need                            | Model                                |
| ------------------------------- | ------------------------------------ |
| inspect caller data during call | borrowed view                        |
| output tied to input            | `*View<'a>` / `*Ref<'a>`             |
| many phase-local objects        | arena                                |
| cheap immutable clones/slices   | shared immutable buffer/string       |
| mutation and transfer           | owned unique value                   |
| mutable graph, stable identity  | typed generational handles           |
| cross-thread immutable read     | `Arc`/frozen snapshot when justified |
| guard-protected shared read     | scoped closure or guard view         |

**Storage:**

| Workload                  | Candidate                       |
| ------------------------- | ------------------------------- |
| 0–N tiny entries          | inline array / linear scan      |
| general local map         | SwissTable-like open addressing |
| read-mostly shared        | epoch/RCU/snapshot              |
| balanced shared           | sharded/fine-grained map        |
| bounded temporary graph   | bump/document arena             |
| streaming bytes           | ring/segmented buffer           |
| repeated immutable slices | shared buffer                   |

**Dispatch:**

| Variability                | Mechanism                     |
| -------------------------- | ----------------------------- |
| compile-time known, hot    | generics/consts               |
| small finite runtime modes | enum + match                  |
| CPU capability             | cached function pointer       |
| plugin/backend boundary    | trait object behind a wrapper |
| rare compatibility mode    | cold dynamic dispatch         |

**Validation:**

| Input                           | Contract                                        |
| ------------------------------- | ----------------------------------------------- |
| public/untrusted                | checked, bounded                                |
| previously validated RustX type | safe fast path, proof in the type               |
| caller-guaranteed raw bytes     | explicit unsafe `*_unchecked`                   |
| internal invariant              | debug assertion + private kernel where measured |

Parser and hash matrices live in [06-parsing-json.md](06-parsing-json.md) and
[07-types-dsl.md](07-types-dsl.md).

## Red flags — absolute

**Architecture:** a second error hierarchy; a crate-specific
buffer/list/string with no justification; circular or upward dependencies;
public APIs shaped around a replaceable backend; a "utils"/"manager" dumping
ground; a public item reachable through competing paths.

**Performance:** speed claimed from source appearance; `target-cpu=native`
benchmarks against generic competitors; SIMD without scalar fallback; CPU
detection in the hot loop; hashing/decoding the same bytes twice without
semantic reason; DOM/hash table where direct consumption suffices; hidden
allocation in a `view`/`as_*` API; parallelism below the amortization
threshold; blanket `#[inline(always)]`; branchless code evaluating expensive
unused arms; caches with unbounded retention; a `Vec` retaining huge one-shot
capacity in a long-lived object without policy.

**Unsafe:** `transmute` as a conversion tool; integer round trips where
strict provenance preserves a pointer; references to uninitialized/invalid
values; `set_len` before initialization; lifetime extension by transmute;
overreading caller slices for SIMD tails; target-feature calls without
detection; manual `Send`/`Sync` without a written theorem;
`unreachable_unchecked` to silence a branch; soundness "established" by tests.

**Concurrency:** atomics with undocumented ordering; lock-free removal
without reclamation; unbounded channels under external load; callbacks or
blocking I/O under locks; synchronous guards across `.await`; a global atomic
on every high-rate operation; a concurrent map as the default local `Dict`;
`Arc<Mutex<_>>` chosen before the thread topology is described; future size
growing unnoticed because a buffer/error crosses `.await`.

**DSL:** Python-likeness via `Box<dyn Any>`; broad conversions hiding clones;
macros swallowing control flow or errors; one facade method whose cost swings
by orders of magnitude with no signal; async behavior behind a
synchronous-looking call; `Cow` where the common path always clones;
`Box<dyn Trait>` introduced to avoid designing a closed enum.

**Dependencies:** direct versions repeated across members; duplicate
capabilities; external types leaking everywhere; reimplementing cryptography
for dependency-count vanity; copying optimized unsafe code without its proofs
and tests; build scripts downloading sources; correctness singletons assuming
one crate version or dylib.

**Agents:** invented benchmark numbers; invented toolchain stability;
rewriting files without inspecting local patterns; running formal gates
unrequested; failing to compile after implementation; a test that copies the
implementation and calls agreement proof; wholesale-pasting an external
source into agent authority without review.
