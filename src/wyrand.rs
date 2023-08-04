#![allow(dead_code)]

use crate::math::{exp2, floor, log, powf};

/// A pseudorandom number generator that uses the WyRand algorithm.
pub struct WyRng {
    /// The current state of the RNG.
    state: u64,
}

/// An iterator over `count` sequential pseudorandom `usize`s in the range `[0, bound)`. Modified
/// from
///
/// https://github.com/shekelyan/sampleiterator/blob/025bf9f963616e996bffa9bf260416a2c8ef9310/hiddenshuffle.rs.
///
/// Shekelyan, M., & Cormode, G. (2021). Sequential Random Sampling Revisited: Hidden Shuffle
/// Method. International Conference on Artificial Intelligence and Statistics.
struct HiddenShuffle<'a> {
    /// The number of high elements.
    high: usize,
    /// The number of low elements.
    low: usize,
    /// The upper bound of the elements in the sequence.
    bound: usize,
    /// The number of elements in the sequence.
    count: usize,
    /// The parameter referred to as alpha in Algorithm 3 of the paper.
    a: f64,
    /// The underlying RNG.
    rng: &'a mut WyRng,
}

impl<'a> HiddenShuffle<'a> {
    fn new(rng: &'a mut WyRng, bound: usize, count: usize) -> Self {
        assert!(bound >= count);

        let mut high: usize = 0;
        let mut i: usize = 0;

        if bound > count {
            high = count;
            while i < count {
                let d = (bound - count) as f64;
                let q = 1.0 - 1.0 * d / (bound - i) as f64;
                i += floor(log(rng.f64(), 1.0 - q)) as usize;
                let pi = 1.0 - 1.0 * d / (bound as f64 - i as f64).max(1.0);
                if i < count && (rng.f64() < (pi / q)) {
                    high -= 1;
                }
                i += 1;
            }
        }

        Self {
            high,
            low: count - high,
            bound,
            count,
            a: 1.0,
            rng,
        }
    }
}

impl<'a> Iterator for HiddenShuffle<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.high > 0 {
            let d = floor((self.bound - self.count) as f64);
            let s_old = self.count + (self.a * d) as usize;
            self.a *= powf(self.rng.f64(), 1.0 / (self.high as f64));
            let s = self.count + (self.a * d) as usize;

            if s < s_old {
                self.high -= 1;
                return Some((self.bound - 1) - s);
            } else {
                self.low += 1; // duplicate detected
                self.high -= 1;
            }
        }

        if self.low > 0 {
            let u = self.rng.f64();
            let mut s = 0;
            let mut f = (self.low as f64) / (self.count as f64);

            while f < u && s < (self.count - self.low) {
                f = 1.0 - (1.0 - (self.low as f64) / ((self.count - s - 1) as f64)) * (1.0 - f);
                s += 1;
            }

            self.low -= 1;
            self.count = self.count - s - 1;

            return Some((self.bound - 1) - self.count);
        }

        None
    }
}

impl WyRng {
    /// Returns a `bool`.
    pub fn bool(&mut self) -> bool {
        self.u64() > u64::MAX / 2
    }

    /// Returns a `f64` in the range `[low, high)`.
    pub fn bounded_f64(&mut self, low: f64, high: f64) -> f64 {
        self.f64() * (high - low) + low
    }

    /// Returns a `u8` in the range `[low, high)`.
    pub fn bounded_u8(&mut self, low: u8, high: u8) -> u8 {
        let range = high - low;
        let mut x = self.u8();
        let mut m = (x as u16) * (range as u16);
        let mut l = m as u8;
        if l < range {
            let mut t = u8::MAX - range;
            if t >= range {
                t -= range;
                if t >= range {
                    t %= range;
                }
            }
            while l < t {
                x = self.u8();
                m = (x as u16) * (range as u16);
                l = m as u8;
            }
        }
        (m >> 8) as u8 + low
    }

    /// Returns a `u16` in the range `[low, high)`.
    pub fn bounded_u16(&mut self, low: u16, high: u16) -> u16 {
        let range = high - low;
        let mut x = self.u16();
        let mut m = (x as u32) * (range as u32);
        let mut l = m as u16;
        if l < range {
            let mut t = u16::MAX - range;
            if t >= range {
                t -= range;
                if t >= range {
                    t %= range;
                }
            }
            while l < t {
                x = self.u16();
                m = (x as u32) * (range as u32);
                l = m as u16;
            }
        }
        (m >> 16) as u16 + low
    }

    /// Returns a `u32` in the range `[low, high)`.
    pub fn bounded_u32(&mut self, low: u32, high: u32) -> u32 {
        let range = high - low;
        let mut x = self.u32();
        let mut m = (x as u64) * (range as u64);
        let mut l = m as u32;
        if l < range {
            let mut t = u32::MAX - range;
            if t >= range {
                t -= range;
                if t >= range {
                    t %= range;
                }
            }
            while l < t {
                x = self.u32();
                m = (x as u64) * (range as u64);
                l = m as u32;
            }
        }
        (m >> 32) as u32 + low
    }

    /// Returns a `u64` in the range `[low, high)`.
    pub fn bounded_u64(&mut self, low: u64, high: u64) -> u64 {
        debug_assert!(low < high);

        let range = high - low;
        let mut x = self.u64();
        let mut m = (x as u128) * (range as u128);
        let mut l = m as u64;
        if l < range {
            let mut t = u64::MAX - range;
            t -= range * (t >= range) as u64;
            if t >= range {
                t %= range;
            }
            while l < t {
                x = self.u64();
                m = (x as u128) * (range as u128);
                l = m as u64;
            }
        }
        (m >> 64) as u64 + low
    }

    /// Returns a `u128` in the range `[low, high)`.
    pub fn bounded_u128(&mut self, low: u128, high: u128) -> u128 {
        let range = high - low;
        let mask = range.next_power_of_two() - 1;
        loop {
            let x = self.u128() & mask;
            if x < range {
                return x + low;
            }
        }
    }

    /// Returns a `usize` in the range `[0, bound)`.
    pub fn bounded_usize(&mut self, low: usize, high: usize) -> usize {
        match core::mem::size_of::<usize>() {
            4 => self.bounded_u32(low as u32, high as u32) as usize,
            8 => self.bounded_u64(low as u64, high as u64) as usize,
            16 => self.bounded_u128(low as u128, high as u128) as usize,
            _ => panic!("Unsupported usize size"),
        }
    }

    /// Returns a `f64` in the range `[0, 1)`.
    pub fn f64(&mut self) -> f64 {
        ((self.u64() >> 11) as f64) * exp2(-53_f64)
    }

    /// Returns a new PRNG initialized with the given seed. If the seed is set to 0, the seed is
    /// based on the address of the PRNG. This should yield an unique sequence for each run of the
    /// program.
    pub fn new(mut seed: u64) -> Self {
        let mut rng = Self { state: 0 };
        #[cfg(not(debug_assertions))]
        if seed == 0 {
            seed = &rng as *const Self as u64;
        }
        #[cfg(debug_assertions)]
        if seed == 0 {
            seed = 123456789123456789;
        }
        rng.state = rng.state.wrapping_add(seed);
        rng
    }

    /// Returns an iterator over `count` sequential pseudorandom `usize`s in the range `[0, bound)`.
    pub fn sequential_usizes(
        &'_ mut self,
        bound: usize,
        count: usize,
    ) -> impl Iterator<Item = usize> + '_ {
        HiddenShuffle::new(self, bound, count)
    }

    /// Returns a `u8`.
    pub fn u8(&mut self) -> u8 {
        (self.u64() >> 56) as u8
    }

    /// Returns a `u16`.
    pub fn u16(&mut self) -> u16 {
        (self.u64() >> 48) as u16
    }

    /// Returns a `u32`.
    pub fn u32(&mut self) -> u32 {
        (self.u64() >> 32) as u32
    }

    /// Returns a `u64`.
    pub fn u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x_a076_1d64_78bd_642f);
        Self::wyrand(self.state)
    }

    /// Returns a `u128`.
    pub fn u128(&mut self) -> u128 {
        (self.u64() as u128) << 64 | self.u64() as u128
    }

    /// Returns a `usize`.
    pub fn usize(&mut self) -> usize {
        match core::mem::size_of::<usize>() {
            4 => self.u32() as usize,
            8 => self.u64() as usize,
            16 => self.u128() as usize,
            _ => panic!("Unsupported usize size"),
        }
    }

    /// Returns a `u64` using `x` as the seed for the wyrand algorithm.
    fn wyrand(x: u64) -> u64 {
        let mut a = x;
        let mut b = x ^ 0x_e703_7ed1_a0b4_28db;
        let r = (a as u128) * (b as u128);
        a = r as u64;
        b = (r >> 64) as u64;
        a ^ b
    }
}

impl AsMut<WyRng> for WyRng {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
