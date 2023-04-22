/// A pseudorandom number generator based on the PCG-XSH-RR-128-64 algorithm.
///
/// See https://www.pcg-random.org/ and https://github.com/imneme/pcg-c/ for more information.
pub struct PCGRng {
    state: u128,
}

impl PCGRng {
    const INCREMENT: u128 = 63641362238467930051442695040888963407_u128;
    const MULTIPLIER: u128 = 25492979953554139244865540595714422341_u128;

    /// Yields a `f32` in the range `[low, high)`.
    pub fn bounded_f32(&mut self, low: f32, high: f32) -> f32 {
        self.f32() * (high - low) + low
    }

    /// Yields a `f64` in the range `[0, bound)`.
    pub fn bounded_f64(&mut self, low: f64, high: f64) -> f64 {
        self.f64() * (high - low) + low
    }

    /// Yields a `i8` in the range `[low, high)`.
    pub fn bounded_i8(&mut self, low: i8, high: i8) -> i8 {
        let range = high.abs_diff(low);
        self.bounded_u8(0, range) as i8 + low
    }

    /// Yields a `i16` in the range `[low, high)`.
    pub fn bounded_i16(&mut self, low: i16, high: i16) -> i16 {
        let range = high.abs_diff(low);
        self.bounded_u16(0, range) as i16 + low
    }

    /// Yields a `i32` in the range `[low, high)`.
    pub fn bounded_i32(&mut self, low: i32, high: i32) -> i32 {
        let range = high.abs_diff(low);
        self.bounded_u32(0, range) as i32 + low
    }

    /// Yields a `i64` in the range `[low, high)`.
    pub fn bounded_i64(&mut self, low: i64, high: i64) -> i64 {
        let range = high.abs_diff(low);
        self.bounded_u64(0, range) as i64 + low
    }

    /// Yields a `i128` in the range `[low, high)`.
    pub fn bounded_i128(&mut self, low: i128, high: i128) -> i128 {
        let range = high.abs_diff(low);
        self.bounded_u128(0, range) as i128 + low
    }

    /// Yields a `isize` in the range `[low, high)`.
    pub fn bounded_isize(&mut self, low: isize, high: isize) -> isize {
        let range = high.abs_diff(low);
        self.bounded_usize(0, range) as isize + low
    }

    /// Yields a `u8` in the range `[low, high)`.
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

    /// Yields a `u16` in the range `[low, high)`.
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

    /// Yields a `u32` in the range `[low, high)`.
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

    /// Yields a `u64` in the range `[low, high)`.
    pub fn bounded_u64(&mut self, low: u64, high: u64) -> u64 {
        let range = high - low;
        let mut x = self.u64();
        let mut m = (x as u128) * (range as u128);
        let mut l = m as u64;
        if l < range {
            let mut t = u64::MAX - range;
            if t >= range {
                t -= range;
                if t >= range {
                    t %= range;
                }
            }
            while l < t {
                x = self.u64();
                m = (x as u128) * (range as u128);
                l = m as u64;
            }
        }
        (m >> 64) as u64 + low
    }

    /// Yields a `u128` in the range `[low, high)`.
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

    /// Yields a `usize` in the range `[0, bound)`.
    pub fn bounded_usize(&mut self, low: usize, high: usize) -> usize {
        match std::mem::size_of::<usize>() {
            4 => self.bounded_u32(low as u32, high as u32) as usize,
            8 => self.bounded_u64(low as u64, high as u64) as usize,
            16 => self.bounded_u128(low as u128, high as u128) as usize,
            _ => panic!("Unsupported usize size"),
        }
    }

    /// Yields a `f32` in the range `[0, 1)`.
    pub fn f32(&mut self) -> f32 {
        ((self.u64() >> 40) as f32) * (-24_f32).exp2()
    }

    /// Yields a `f64` in the range `[0, 1)`.
    pub fn f64(&mut self) -> f64 {
        ((self.u64() >> 11) as f64) * (-53_f64).exp2()
    }

    /// Yields an `i8`.
    pub fn i8(&mut self) -> i8 {
        self.u8() as i8
    }

    /// Yields an `i16`.
    pub fn i16(&mut self) -> i16 {
        self.u16() as i16
    }

    /// Yields an `i32`.
    pub fn i32(&mut self) -> i32 {
        self.u32() as i32
    }

    /// Yields an `i64`.
    pub fn i64(&mut self) -> i64 {
        self.u64() as i64
    }

    /// Yields an `i128`.
    pub fn i128(&mut self) -> i128 {
        self.u128() as i128
    }

    /// Yields an `isize`.
    pub fn isize(&mut self) -> isize {
        self.usize() as isize
    }

    /// Applies the provided function to the PRNG and returns the result.
    pub fn map<T>(&mut self, func: fn(&mut Self) -> T) -> T {
        (func)(self)
    }

    /// Returns a new PRNG initialized with the given seed. If the seed is set to 0, the seed is
    /// based on the address of the PRNG. This should yield an unique sequence for each run of the
    /// program.
    pub fn new(mut seed: u64) -> Self {
        let mut rng = Self { state: 0 };
        if seed == 0 {
            seed = &rng as *const Self as u64;
        }
        rng.step();
        rng.state = rng.state.wrapping_add(seed as u128);
        rng.step();
        rng
    }

    /// Generates the next value from the current state of the PRNG.
    fn output(&self) -> u64 {
        let value = (((self.state >> 35) ^ self.state) >> 58) as u64;
        let rot = (self.state >> 122) as u32;
        (value >> rot) | (value << ((u32::MAX - rot) & 63))
    }

    /// Advances the state of the PRNG.
    fn step(&mut self) {
        self.state = self
            .state
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::INCREMENT)
    }

    /// Yields a `u8`.
    pub fn u8(&mut self) -> u8 {
        (self.u64() >> 56) as u8
    }

    /// Yields a `u16`.
    pub fn u16(&mut self) -> u16 {
        (self.u64() >> 48) as u16
    }

    /// Yields a `u32`.
    pub fn u32(&mut self) -> u32 {
        (self.u64() >> 32) as u32
    }

    /// Yields a `u64`.
    pub fn u64(&mut self) -> u64 {
        self.step();
        self.output()
    }

    /// Yields a `u128`.
    pub fn u128(&mut self) -> u128 {
        (self.u64() as u128) << 64 | self.u64() as u128
    }

    /// Yields a `usize`.
    pub fn usize(&mut self) -> usize {
        match std::mem::size_of::<usize>() {
            4 => (self.u64() >> 32) as usize,
            8 => self.u64() as usize,
            16 => ((self.u64() as u128) << 64 | self.u64() as u128) as usize,
            _ => panic!("Unsupported usize size"),
        }
    }
}
