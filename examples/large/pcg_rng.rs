/// A pseudorandom number generator based on the PCG-XSH-RR-128-64 algorithm.
///
/// See https://www.pcg-random.org/ and https://github.com/imneme/pcg-c/ for more information.
pub struct PCGRng {
    state: u128,
}

impl PCGRng {
    const INCREMENT: u128 = 63641362238467930051442695040888963407_u128;
    const MULTIPLIER: u128 = 25492979953554139244865540595714422341_u128;

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
}
