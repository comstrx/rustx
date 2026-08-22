# Unsafe Rust — A Proof Discipline

**Load when:** raw pointers, `MaybeUninit`, custom allocation, SIMD loads, FFI,
atomics, or unsafe auto-trait implementations appear.

**Output:** a complete safety case, state-transition proof, and safe facade.

## Unsafe is a proof obligation, not permission

The only valid reason for `unsafe` is that Rust cannot express or verify an
invariant which the implementation can prove and preserve. Valid capability
classes: novel abstractions (smart pointers, allocators), measured hot-path
performance, FFI/platform calls. Invalid reasons: "the borrow checker is
annoying," "this should be faster" without generated-code evidence, "the input
is probably valid," "the pointer came from the same address," "tests pass."

Never use ad-hoc unsafe to shorten a safe program, bypass `Send`/lifetimes via
`transmute`, or silence a bound. Workspace lints already deny undocumented
unsafe blocks, multiple ops per block, and missing safety docs — treat those as
the floor, not the ceiling.

## The unsafe capsule

```text
safe public facade
    ↓ validates / establishes invariants
private unsafe kernel
    ↓ performs the minimal unchecked operation
safe result with restored invariants
```

Raw pointers never leak across unrelated modules. Recommended kernel module
shape:

```text
kernel/
├── invariant.md      # representation and state invariants
├── kernel.rs         # safe facade / dispatch
├── scalar.rs         # oracle and fallback
├── arch_x86.rs       # target-feature kernels
└── tests.rs          # differential and boundary assets
```

A `// SAFETY:` comment explains one operation. The module safety case explains
why all operations compose.

## The mandatory safety case

Every non-trivial unsafe fn, unsafe trait impl, or unsafe block has an
adjacent safety case answering:

```text
SAFETY:
- Validity: which bytes/values are initialized and valid for T?
- Bounds: which allocation contains every access?
- Alignment: why is each pointer aligned, or read unaligned?
- Provenance: from which allocation was each pointer derived?
- Aliasing: which shared/mutable references exist during the operation?
- Lifetime: why can no pointer/reference outlive storage?
- Concurrency: what synchronization establishes happens-before?
- Panic: what remains valid if an operation unwinds here?
- Drop: who destroys each initialized value exactly once?
- CPU: which target features are required and how were they checked?
```

"Safe because pointer is valid" is not a safety case.

## Unsafe state-transition table

For stateful unsafe code, write the transitions; if a state cannot be
described, the representation is not ready for unsafe implementation:

| State     | Initialized | Owned by  | May alias   | Valid next         | Panic/cancel action      |
| --------- | ----------- | --------- | ----------- | ------------------ | ------------------------ |
| allocated | 0..0        | builder   | unique raw  | partial, committed | deallocate only          |
| partial   | 0..init     | guard     | unique raw  | partial, committed | drop prefix + deallocate |
| committed | 0..len      | container | API-defined | moved, dropped     | normal drop              |

## References are stronger than pointers

Creating `&T`/`&mut T` asserts validity, alignment, lifetime, and aliasing
immediately — even if never read. Keep raw pointers raw until every reference
invariant holds. Use `&raw const` / `&raw mut` when forming a pointer to a
field could otherwise create an invalid intermediate reference.

## Strict provenance by default

Prefer pointer-preserving operations — `addr`, `with_addr`, `map_addr` — over
pointer→integer→pointer round trips. Represent tagged pointers as pointers,
not bare `usize`. Use `AtomicPtr<T>`, never smuggle pointers through
`AtomicUsize`. Exposed provenance is an explicit FFI/MMIO escape hatch,
isolated, never the default model.

## Aliasing doctrine

A live `&mut T` is exclusive for its reachable region except through designed
`UnsafeCell` interiors. Never create two mutable references to overlapping
memory, even "used one at a time." Never mutate through a raw pointer while a
forbidding shared reference is live. Split slices with proven non-overlap
before parallel mutation. Prefer indices/handles when mutable graph topology
makes references difficult.

## `MaybeUninit` doctrine

Uninitialized memory is `MaybeUninit<T>`, never fake `T`. Zero bytes are not
universally valid. Never read, drop, compare, or reference an uninitialized
value. Track partial initialization exactly; conversion to `T` is the final
commit, not a hopeful assertion. Never `mem::uninitialized()`/`mem::zeroed()`
for types with validity invariants.

## `ManuallyDrop` doctrine

It changes automatic destruction — not validity, aliasing, ownership, or
initialization. Prevent double-drop through state, not comments. Never expose
a safe API that observes a logically dropped value.

## Panic and cancellation safety

Unsafe code preserves memory safety at every unwind point; safe code preserves
logical consistency. Use transactions (prepare → fallible work → one
invariant-changing commit) or drop guards that roll back. Async cancellation
is an unwind-like boundary: a future may be dropped at any `.await` — never
leave half-published nodes or inconsistent protocol frames.

## `Send`/`Sync` are public theorems

A manual `unsafe impl Send/Sync` without a state-by-state proof is forbidden.
The proof covers: raw pointers and pointee lifetime, interior mutability,
thread affinity, destruction thread, allocator/pool ownership, guard and
reclamation model, captured callbacks. Prefer letting the compiler derive
auto traits; encode deliberate thread-confinement intentionally.

## Adversarial hardening (for novel abstractions)

- If the abstraction accepts closures, it must become invalid (e.g., poisoned)
  when the closure panics.
- Assume any safe trait may misbehave — especially `Deref`, `Clone`, `Drop`.
- Layout size/alignment assertions detect regressions; they do not create a
  stable ABI. Never transmute because sizes match — the workspace bans
  `transmute` outright via disallowed methods; safe conversions or documented
  pointer casts only.

## The unsafe removal test

Before accepting unsafe, check whether one of these removes it at negligible
cost: indices/handles instead of self-references; split borrows / slice APIs;
std's `MaybeUninit` helpers; typestate or phase separation; one strategic
copy; scoped closure access; a safe dependency with a stronger proof history.
Unsafe is justified by a concrete capability or a measured hot-path delta —
never aesthetics.

## Rust 2024 syntax obligations

Wrap FFI in `unsafe extern { }` with each item marked `safe`/`unsafe`. Write
`#[unsafe(no_mangle)]`, `#[unsafe(export_name = "…")]`,
`#[unsafe(link_section = "…")]` — never the bare forms. `unsafe fn` bodies
require explicit inner `unsafe {}` blocks (`unsafe_op_in_unsafe_fn` is denied).

## Verification assets

When gates are requested for unsafe code, select the relevant set: Miri
(including adversarial cases) for the UB classes it models; differential tests
against the scalar oracle; fuzzing and property tests; sanitizers for
native/FFI paths; Loom or a model checker for synchronization; cross-arch
execution; panic injection around invariant transitions; layout assertions;
an audit of every unsafe call site. Passing Miri is evidence, not proof;
failing Miri is a defect until proven otherwise.
