/// A pseudorandom number generator that uses the WyRand algorithm.
pub struct WyRng {
    /// The current state of the RNG.
    state: u64,
}

impl WyRng {
    /// Returns a mutable reference to the RNG.
    pub fn as_mut(&mut self) -> &mut Self {
        self
    }

    /// Returns a `bool`.
    pub fn bool(&mut self) -> bool {
        self.u64() > u64::MAX / 2
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

    /// Returns a `u64` using `x` as the seed for the wyrand algorithm.
    pub fn wyrand(x: u64) -> u64 {
        let mut a = x;
        let mut b = x ^ 0x_e703_7ed1_a0b4_28db;
        let r = (a as u128) * (b as u128);
        a = r as u64;
        b = (r >> 64) as u64;
        a ^ b
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
}
