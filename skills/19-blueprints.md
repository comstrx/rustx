# Reference Blueprints

**Load when:** implementing one of these exact mechanisms. Adapt each
blueprint to the crate's vocabulary — never copy blindly; every blueprint
carries invariants the surrounding code must uphold.

## 1. Panic-safe `MaybeUninit` initialization guard

Transactional bulk initialization: success appends every value; panic drops
exactly the initialized prefix and leaves the vector unchanged.

```rust
use std::mem::MaybeUninit;

struct SpareInitGuard<'a, T> {
    spare: &'a mut [MaybeUninit<T>],
    initialized: usize,
    committed: bool,
}

impl<T> SpareInitGuard<'_, T> {
    #[inline]
    fn write_next(&mut self, value: T) {
        debug_assert!(self.initialized < self.spare.len());
        self.spare[self.initialized].write(value);
        self.initialized += 1;
    }

    #[inline]
    fn commit(mut self) -> usize {
        self.committed = true;
        self.initialized
    }
}

impl<T> Drop for SpareInitGuard<'_, T> {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        for slot in &mut self.spare[..self.initialized] {
            // SAFETY: `write_next` initializes the prefix exactly once and
            // increments `initialized` only after the write succeeds.
            unsafe { slot.assume_init_drop() }
        }
    }
}

pub fn extend_exact<T>(vec: &mut Vec<T>, additional: usize, mut next: impl FnMut(usize) -> T) {
    let original_len = vec.len();
    vec.reserve(additional);

    let initialized = {
        let spare = &mut vec.spare_capacity_mut()[..additional];
        let mut guard = SpareInitGuard { spare, initialized: 0, committed: false };
        for index in 0..additional {
            guard.write_next(next(index));
        }
        guard.commit()
    };

    debug_assert_eq!(initialized, additional);

    // SAFETY: `reserve` provided the capacity and the guard committed exactly
    // the appended range as initialized.
    unsafe { vec.set_len(original_len + initialized) }
}
```

An API that intentionally preserves the initialized prefix after a panic is a
different named contract with a set-length-on-drop guard.

## 2. Cached SIMD dispatch

Detection once, cached function pointer forever, scalar truth preserved,
kernels private so no unchecked path bypasses the CPU invariant.

```rust
use std::sync::OnceLock;

type FindByteFn = fn(&[u8], u8) -> Option<usize>;

static FIND_BYTE: OnceLock<FindByteFn> = OnceLock::new();

#[inline]
pub fn find_byte(haystack: &[u8], needle: u8) -> Option<usize> {
    (FIND_BYTE.get_or_init(resolve_find_byte))(haystack, needle)
}

#[cold]
fn resolve_find_byte() -> FindByteFn {
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx2") {
            // SAFETY: the AVX2 requirement of `find_byte_avx2` was just
            // established by runtime detection, once, for process lifetime.
            return |haystack, needle| unsafe { find_byte_avx2(haystack, needle) };
        }
    }
    find_byte_scalar
}

fn find_byte_scalar(haystack: &[u8], needle: u8) -> Option<usize> {
    haystack.iter().position(|&byte| byte == needle)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn find_byte_avx2(haystack: &[u8], needle: u8) -> Option<usize> {
    // Private kernel. Must preserve scalar semantics exactly (differential
    // tests against `find_byte_scalar` are the oracle).
    find_byte_scalar(haystack, needle)
}
```

## 3. Guard-hiding concurrent access

The closure compresses lifetime complexity: `&V` cannot escape the epoch/lock
protection, without allocating or cloning.

```rust
pub trait SharedLookup<K, V> {
    fn with<R>(&self, key: &K, use_value: impl for<'guard> FnOnce(Option<&'guard V>) -> R) -> R;
}
```

An implementation may pin an epoch or hold a shard guard internally; the
higher-rank bound guarantees the borrow dies inside the call.

## 4. Typestate facade

Invalid calls removed at compile time; the state parameter hidden behind the
builder flow.

```rust
pub struct Unconfigured;
pub struct Ready;

pub struct Parser<State> {
    plan: ParserPlan,
    _state: core::marker::PhantomData<State>,
}

impl Parser<Unconfigured> {
    #[must_use]
    pub fn new() -> Self {
        Self { plan: ParserPlan::default(), _state: core::marker::PhantomData }
    }

    #[must_use]
    pub fn limits(mut self, limits: Limits) -> Self {
        self.plan.limits = limits;
        self
    }

    pub fn build(self) -> Result<Parser<Ready>, Error> {
        self.plan.validate()?;
        Ok(Parser { plan: self.plan, _state: core::marker::PhantomData })
    }
}

impl Parser<Ready> {
    pub fn parse<'a>(&self, input: &'a [u8]) -> Result<JsonRef<'a>, Error> {
        self.plan.parse(input)
    }
}
```

Alias or wrap the states if displaying them would harm ergonomics.

## 5. Strict/algebraic numeric kernel

Two explicit numerical modes; the mode resolved outside the inner loop into a
specialized plan — never a branch per operation, never a silent substitution.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FloatMode {
    /// IEEE-sensitive operation ordering; deterministic as far as the
    /// platform permits.
    Strict,
    /// Rust 1.98 algebraic operations: reassociation and vectorization
    /// permitted; rounding, NaN, signed-zero, and determinism may change.
    Algebraic,
}

#[inline]
#[must_use]
pub fn sum_f32(values: &[f32], mode: FloatMode) -> f32 {
    match mode {
        FloatMode::Strict => values.iter().sum(),
        FloatMode::Algebraic => values.iter().fold(0.0, |acc, &v| acc.algebraic_add(v)),
    }
}
```

## 6. Cold structured error

Success path small and inlined; failure construction outlined and cold.

```rust
#[inline]
pub fn decode(input: &[u8]) -> Result<Value, Error> {
    match decode_fast(input) {
        Some(value) => Ok(value),
        None => decode_error(input),
    }
}

#[cold]
#[inline(never)]
fn decode_error(input: &[u8]) -> Result<Value, Error> {
    core::hint::cold_path();
    Err(Error::new(ErrorCode::INVALID_INPUT).with_offset(find_failure_offset(input)))
}
```
