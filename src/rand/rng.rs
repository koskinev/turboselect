use super::PCGRng;

/// A pseudorandom number generator based on the PCG-XSH-RR-128-64 algorithm.
///
/// A `MappedRng` is constructed by providing a seed and a function that takes a mutable reference
/// to a `PCGRng` and returns a value of type `T`. This function may call the methods provided
/// by the `PCGRng` struct to generate random data to construct the return value.
pub struct MappedRng<T> {
    /// The underlying PRNG.
    prng: PCGRng,
    /// A function that yields a value of type `T` from a `PCGRng`.
    map: fn(&mut PCGRng) -> T,
    /// A function that yields a bounded value of type `T` from a `PCGRng`. The function should
    /// return a value in the range specified by the second and third arguments.
    bound_map: fn(&mut PCGRng, T, T) -> T,
}

impl<T> MappedRng<T> {
    /// Binds the values of the RNG to the range `[low, high)`.
    pub fn in_range(self, low: T, high: T) -> BoundedRng<T> {
        BoundedRng {
            rng: self,
            low,
            high,
        }
    }

    /// Gets a value from the sequence.
    pub fn get(&mut self) -> T {
        (self.map)(&mut self.prng)
    }

    /// Gets a value bounded by the given lower and upper bounds.
    pub fn get_bounded(&mut self, low: T, high: T) -> T {
        (self.bound_map)(&mut self.prng, low, high)
    }
}

impl<T> Iterator for MappedRng<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        Some(self.get())
    }
}

/// A pseudo-random number generator that yields values in a bounded range. See the documentation
/// for `MappedRng` for more information.
pub struct BoundedRng<T> {
    rng: MappedRng<T>,
    low: T,
    high: T,
}

impl<T> BoundedRng<T>
where
    T: Copy,
{
    /// Gets a value from the sequence.
    pub fn get(&mut self) -> T {
        self.rng.get_bounded(self.low, self.high)
    }
}

impl<T> Iterator for BoundedRng<T>
where
    T: Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        Some(self.rng.get_bounded(self.low, self.high))
    }
}

/// An extension trait for generating random values. This trait is implemented for `u8`, `u16`,
/// `u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128`, `f32`, and `f64` and for arrays of
/// these types.
pub trait Rng: Sized {
    const MAP: fn(&mut PCGRng) -> Self;

    const BOUND_MAP: fn(&mut PCGRng, Self, Self) -> Self;

    /// Constructs a random number generator that yields values of type `Self`. If `seed` is
    /// set to `0`, the address of the generator will be used as a seed.     
    fn rng(seed: u64) -> MappedRng<Self> {
        MappedRng {
            prng: PCGRng::new(seed),
            map: Self::MAP,
            bound_map: Self::BOUND_MAP,
        }
    }
}

impl Rng for u8 {
    const BOUND_MAP: fn(&mut PCGRng, Self, Self) -> Self =
        |prng, low, high| prng.bounded_u8(low, high);
    const MAP: fn(&mut PCGRng) -> Self = |prng| prng.u8();
}

impl Rng for u16 {
    const BOUND_MAP: fn(&mut PCGRng, Self, Self) -> Self =
        |prng, low, high| prng.bounded_u16(low, high);
    const MAP: fn(&mut PCGRng) -> Self = |prng| prng.u16();
}

impl Rng for u32 {
    const BOUND_MAP: fn(&mut PCGRng, Self, Self) -> Self =
        |prng, low, high| prng.bounded_u32(low, high);
    const MAP: fn(&mut PCGRng) -> Self = |prng| prng.u32();
}

impl Rng for u64 {
    const BOUND_MAP: fn(&mut PCGRng, Self, Self) -> Self =
        |prng, low, high| prng.bounded_u64(low, high);
    const MAP: fn(&mut PCGRng) -> Self = |prng| prng.u64();
}

impl Rng for u128 {
    const BOUND_MAP: fn(&mut PCGRng, Self, Self) -> Self =
        |prng, low, high| prng.bounded_u128(low, high);
    const MAP: fn(&mut PCGRng) -> Self = |prng| prng.u128();
}

impl Rng for usize {
    const BOUND_MAP: fn(&mut PCGRng, Self, Self) -> Self =
        |prng, low, high| prng.bounded_usize(low, high);
    const MAP: fn(&mut PCGRng) -> Self = |prng| prng.usize();
}

impl Rng for i8 {
    const BOUND_MAP: fn(&mut PCGRng, Self, Self) -> Self =
        |prng, low, high| prng.bounded_i8(low, high);
    const MAP: fn(&mut PCGRng) -> Self = |prng| prng.i8();
}

impl Rng for i16 {
    const BOUND_MAP: fn(&mut PCGRng, Self, Self) -> Self =
        |prng, low, high| prng.bounded_i16(low, high);
    const MAP: fn(&mut PCGRng) -> Self = |prng| prng.i16();
}

impl Rng for i32 {
    const BOUND_MAP: fn(&mut PCGRng, Self, Self) -> Self =
        |prng, low, high| prng.bounded_i32(low, high);
    const MAP: fn(&mut PCGRng) -> Self = |prng| prng.i32();
}

impl Rng for i64 {
    const BOUND_MAP: fn(&mut PCGRng, Self, Self) -> Self =
        |prng, low, high| prng.bounded_i64(low, high);
    const MAP: fn(&mut PCGRng) -> Self = |prng| prng.i64();
}

impl Rng for i128 {
    const BOUND_MAP: fn(&mut PCGRng, Self, Self) -> Self =
        |prng, low, high| prng.bounded_i128(low, high);
    const MAP: fn(&mut PCGRng) -> Self = |prng| prng.i128();
}

impl Rng for isize {
    const BOUND_MAP: fn(&mut PCGRng, Self, Self) -> Self =
        |prng, low, high| prng.bounded_isize(low, high);
    const MAP: fn(&mut PCGRng) -> Self = |prng| prng.isize();
}

impl Rng for f32 {
    const BOUND_MAP: fn(&mut PCGRng, Self, Self) -> Self =
        |prng, low, high| prng.bounded_f32(low, high);
    const MAP: fn(&mut PCGRng) -> Self = |prng| prng.f32();
}

impl Rng for f64 {
    const BOUND_MAP: fn(&mut PCGRng, Self, Self) -> Self =
        |prng, low, high| prng.bounded_f64(low, high);
    const MAP: fn(&mut PCGRng) -> Self = |prng| prng.f64();
}

impl<const D: usize, T> Rng for [T; D]
where
    T: Rng + Copy,
{
    const BOUND_MAP: fn(&mut PCGRng, Self, Self) -> Self = |prng, low, high| {
        std::array::from_fn(|index| (T::BOUND_MAP)(prng, low[index], high[index]))
    };
    const MAP: fn(&mut PCGRng) -> Self = |prng| std::array::from_fn(|_| (T::MAP)(prng));
}
