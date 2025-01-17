// The constants used by the Linear Congruential Generator
pub mod lcg_const {
    pub const A: u64 = 0x5DEECE66D;
    pub const C: u64 = 0xB;
}

// Constants used to reverse operations
pub mod lcg_const_extra {
    pub const INV_A: u64 = 0xdfe05bcb1365;
    pub const INV_A_1: u64 = 18698324575379;
    pub const INV__INV_A__1: u64 = 192407907957609;
}

/// Return a mask which will keep the lower n bits
/// ```
/// use slime_seed_finder::java_rng::mask;
///
/// let fifteen = 0b1111;
/// assert_eq!(fifteen & mask(0), 0b0000);
/// assert_eq!(fifteen & mask(1), 0b0001);
/// assert_eq!(fifteen & mask(2), 0b0011);
/// assert_eq!(fifteen & mask(3), 0b0111);
/// assert_eq!(fifteen & mask(4), 0b1111);
/// ```
pub const fn mask(n: u8) -> u64 {
    (1 << n) - 1
}

#[derive(Copy, Clone, Debug)]
pub struct JavaRng {
    // Actually only 48 bits of seed are used in java
    // but we use 64 internally, masking only when needed
    seed: u64,
}

impl JavaRng {
    pub fn with_seed(s: u64) -> JavaRng {
        let mut r = JavaRng { seed: 0 };
        r.set_seed(s);
        r
    }

    pub fn with_raw_seed(s: u64) -> JavaRng {
        let mut r = JavaRng { seed: 0 };
        r.set_raw_seed(s);
        r
    }

    pub fn set_seed(&mut self, s: u64) {
        self.seed = s ^ lcg_const::A;
    }

    pub fn set_raw_seed(&mut self, s: u64) {
        self.seed = s;
    }

    pub fn get_seed(&self) -> u64 {
        (self.seed ^ lcg_const::A) & mask(48)
    }

    pub fn get_raw_seed(&self) -> u64 {
        self.seed & mask(48)
    }

    pub fn next(&mut self, bits: u8) -> i32 {
        self.seed = Self::next_state(self.seed);
        ((self.seed & mask(48)) >> (48 - bits)) as i32
    }

    // s * A + C
    pub fn next_state(s: u64) -> u64 {
        s.wrapping_mul(lcg_const::A).wrapping_add(lcg_const::C)
    }

    // Returns the same as the last call to next
    pub fn last_next(&self, bits: u8) -> i32 {
        ((self.seed & mask(48)) >> (48 - bits)) as i32
    }

    // (s * A) + C
    // ((s * A) + C) * A + C
    // s*A*A + C*A + C
    // s*A*A*A + C*A*A + C*A + C
    // Equivalent to n calls to next
    pub fn next_n_calls(&mut self, n: u64) {
        // I doubt this function will ever be useful, but
        // at least I had fun making it.
        match n {
            0 => return,
            1 => {
                self.next(0);
                return;
            }
            _ => {}
        }
        let c = lcg_const::C;
        let a = lcg_const::A;
        // Modular multiplicative inverse of a-1
        let a_1_inv = lcg_const_extra::INV_A_1;
        let an = pow_wrapping(a, n);
        //let aes = (an - 1) / (a - 1);
        // a % 4 == 1, so (a^n - 1) % 4 == 0
        let aes = (an.wrapping_sub(1) >> 2).wrapping_mul(a_1_inv);
        let caa = c.wrapping_mul(aes);
        self.seed = self.seed.wrapping_mul(an).wrapping_add(caa);
    }

    pub fn next_int(&mut self) -> i32 {
        self.next(32)
    }

    pub fn next_int_n(&mut self, n: i32) -> i32 {
        if n == 10 {
            return self.next_int_n_10();
        }
        if !(n > 0) {
            panic!("In JavaRng::next_int_n, n should be greater than zero.");
        }
        // If n is a power of 2
        if (n & -n) == n {
            return (((n as i64) * (self.next(31) as i64)) >> 31) as i32;
        }

        let mut bits;
        let mut val;
        loop {
            bits = self.next(31);
            val = bits % n;
            // Check for modulo bias
            if bits.wrapping_sub(val).wrapping_add(n - 1) >= 0 {
                break;
            }
        }

        val
    }

    pub fn next_int_n_10(&mut self) -> i32 {
        let mut bits;
        loop {
            bits = self.next(31);
            // Check for modulo bias
            let limit = (1u32 << 31) / 10 * 10; // last multiple of 10 < 2^31
            if bits < limit as i32 {
                break;
            }
        }
        bits % 10
    }

    pub fn next_long(&mut self) -> i64 {
        ((self.next_int() as i64) << 32) + (self.next_int() as i64)
    }

    pub fn next_boolean(&mut self) -> bool {
        self.next(1) != 0
    }

    pub fn next_float(&mut self) -> f32 {
        self.next(24) as f32 / (1 << 24) as f32
    }

    pub fn next_double(&mut self) -> f64 {
        let hi = (self.next(26) as i64) << 27;
        let lo = self.next(27) as i64;

        (hi + lo) as f64 / ((1u64 << 53) as f64)
    }

    // The inverse of next()
    pub fn previous(&mut self) {
        //self.seed = (self.seed.wrapping_sub(lcg_const::C)).wrapping_mul(lcg_const_extra::INV_A);
        self.seed = Self::previous_state(self.seed);
    }

    // The previous internal state of the prng, not the seed
    pub fn previous_state(s: u64) -> u64 {
        (s.wrapping_sub(lcg_const::C)).wrapping_mul(lcg_const_extra::INV_A) & mask(48)
    }

    // Equivalent to 2 calls to previous
    /* (((s - C) * D) - C) * D =
     * (s*D - C*D - C) * D =
     * s*D*D - C*D*D - C*D =
     * s*(D*D) - (C*D*(D+1)) */
    pub fn previous_2(&mut self) {
        /* It seems that the compiler isn't that smart after all
        self.previous();
        self.previous();
        */
        //self.seed = JavaRng::previous_state(JavaRng::previous_state(self.seed));
        self.seed = (((self.seed.wrapping_sub(lcg_const::C)).wrapping_mul(lcg_const_extra::INV_A))
            .wrapping_sub(lcg_const::C))
        .wrapping_mul(lcg_const_extra::INV_A);
    }

    /* 3 calls:
     * (s*(D*D) - (C*D*(D+1)) - C) * D
     * s*(D*D*D) - (C*D*(D+1))*D - C*D
     * s*(D*D*D) - (C*D*D*D + C*D*D + C*D)
     * s*(D*D*D) - (C*D*(D*D+D+1) */
    /* n calls:
     * s*(D**n) - C*D*(D**(n-1) + ... + D**2 + D**1)
     * s*(D**n) is trivial, but what about the sum of powers of D?
     * S = D**(n-1) + ... + D**0
     * S * D = D**n + ... + D**1
     * S * D + D**0 = D**n + S
     * S * (D - 1) = D**n - D**0
     * S = (D**n - 1) / (D - 1) */
    pub fn previous_n_calls(&mut self, n: u64) {
        // I doubt this function will ever be useful, but
        // at least I had fun making it.
        match n {
            0 => return,
            1 => return self.previous(),
            2 => return self.previous_2(),
            _ => {}
        }
        let c = lcg_const::C;
        let d: u64 = lcg_const_extra::INV_A;
        // Modular multiplicative inverse of d-1
        let d_1_inv = lcg_const_extra::INV__INV_A__1;
        let dn = pow_wrapping(d, n);
        //let des = (dn - 1) / (d - 1);
        let des = (dn.wrapping_sub(1) >> 2).wrapping_mul(d_1_inv);
        let cdd = c.wrapping_mul(d).wrapping_mul(des);
        self.seed = self.seed.wrapping_mul(dn).wrapping_sub(cdd);
    }

    pub fn previous_verify_16(&self, target: u16) -> u32 {
        let p1 = Self::previous_state(self.seed) as u16;
        let p = ((target as u32) << 16) | (p1 as u32);
        let p2 = p
            .wrapping_mul((lcg_const::A & mask(32)) as u32)
            .wrapping_add(lcg_const::C as u32);

        p2 ^ (self.seed as u32)
    }

    pub fn previous_verify_n(&self, target: u64, mut n: u8) -> u64 {
        if n > 48 {
            n = 48;
        }
        let p1 = Self::previous_state(self.seed) as u16;
        let p = ((target as u64) << 16) | (p1 as u64);
        let p2 = Self::next_state(p);

        (p2 ^ self.seed) & mask(n)
    }

    pub fn i1_from_long(l: i64) -> i32 {
        l as i32
    }

    pub fn i0_from_long(l: i64) -> i32 {
        ((l >> 32) + ((l >> 31) & 1)) as i32
    }

    pub fn ints_from_long(l: i64) -> (i32, i32) {
        (Self::i0_from_long(l), Self::i1_from_long(l))
    }

    pub fn long_from_i0_i1(i0: i32, i1: i32) -> i64 {
        ((i0 as i64) << 32) + (i1 as i64)
    }

    // Suppose we call r.next_int() and obtain i0. If we want to obtain i1 in
    // the next call, what should be the value of the lower 16 bits of the
    // internal seed? (the higher 32 are already known to be i0)
    //
    // Derivation:
    //
    // Since i1 = (((i0 << 16) + low16) * A + C) >> 16
    // And C >> 16 can be only 0 or sometimes 1, we set it to 0
    // i1 = (((i0 << 16) + low16) * A) >> 16
    // Illegal parenthesis manipulation
    // i1 = ((i0 << 16) + low16) * (A >> 16)
    // i1 = i0 * A + low16 * (A >> 16)
    // i1 - i0 * A = low16 * (A >> 16)
    // (i1 - i0 * A) / (A >> 16) = low16
    //
    // We want the result of the division to be greater than 2^16, so that the
    // low 16 bits are correct.
    //
    // i0 and i1 are 32-bit values, A is a 35-bit value.
    // (i1 - i0 * A) / (A >> 16)
    // From the left side of the `/` we get 32 bits, but from the right side we
    // get (35 - 16) = 19 bits. Since we divide a 32-bit value by a 19-bit value
    // the result is a (32 - 19) = 13 bit value.
    //
    // So we can only extract 13 bits from here.
    //
    // But that's not a problem, we can just bruteforce the remaining 3 bits.
    //
    // In fact, we only need to bruteforce 2.58 bits (6 values instead of 8)
    // because the constant A is not exactly 2^35, its slightly below
    // 2^34 + 2^33. So in the worst case we need to check 6 extra values.
    // That's a great improvement over the naive 2^16.
    pub fn low_16_for_next_int(i0: u32, i1: u32) -> Option<u16> {
        // x = i1 - i0 * A
        let x: u32 = i1.wrapping_sub(i0.wrapping_mul((lcg_const::A & mask(32)) as u32));
        for i in 0..6 {
            let low16 = ((x as u64 | ((i as u64) << 32)) / (lcg_const::A >> 16)) as u16;
            let y = (Self::next_state(low16 as u64) >> 16) as u32;
            if y == x {
                // Could this function return more than one value?
                // No, because this PRNG has no loops
                return Some(low16);
            }
        }

        None
    }

    // Returns a JavaRng r such that r.next_long() will return l
    pub fn create_from_long(l: u64) -> Option<JavaRng> {
        let (i0, i1) = Self::ints_from_long(l as i64);
        let (i0, i1) = (i0 as u32, i1 as u32);
        let front = (i0 as u64) << 16;
        if let Some(back) = Self::low_16_for_next_int(i0, i1) {
            let mut r = JavaRng::with_raw_seed(front | (back as u64));
            r.previous();
            Some(r)
        } else {
            None
        }
    }

    // We have the lower 48 bits of r.next_long(), what are the other bits?
    // This function can return more than one number! Sometimes 0, sometimes 2
    pub fn extend_long_48(l: u64) -> Vec<u64> {
        let l = l & mask(48);
        let (i0, i1) = JavaRng::ints_from_long(l as i64);
        let i0 = i0 as u16;
        let seed = ((i1 as u32 as u64) << 16) & mask(48);

        (0..0x10000) // for every 16 bit number
            .into_iter()
            .map(|k0| {
                let s = seed | (k0 as u64);

                JavaRng::with_raw_seed(s)
            })
            .filter(|r| r.previous_verify_16(i0) == 0)
            .map(|mut r| {
                r.previous();
                r.previous();

                r.next_long() as u64
            })
            .collect()
    }

    /// How many calls to `next` are needed to transform self into other
    pub fn num_steps_to(&self, other: &Self) -> u64 {
        distance_between_rngs(self, other)
    }

    /// Returns how many calls to `next` are needed to transform self into other, or `None` if the
    /// number of calls is greater than or equal to `l`
    pub fn num_steps_to_other_under_l(&self, other: &Self, l: u64) -> Option<u64> {
        distance_between_rngs_less_than(self, other, l)
    }
}

// Calculate base^exp (mod 2^64).
// Copied from the standard library, but the wrapping_pow implemented there uses u32 for the
// exponent. We could use some property like a^(b*(2^32) + c) = ((a^b)^(2^32)) * (a^c)
// But just copying the implementation and changing one type seems easier
// https://github.com/rust-lang/rust/blob/118b50524b79e565f017e08bce9b90a16c63634f/src/libcore/num/mod.rs#L1611
fn pow_wrapping(mut base: u64, mut exp: u64) -> u64 {
    let mut acc: u64 = 1;

    while exp > 1 {
        if (exp & 1) == 1 {
            acc = acc.wrapping_mul(base);
        }
        exp /= 2;
        base = base.wrapping_mul(base);
    }

    // Deal with the final bit of the exponent separately, since
    // squaring the base afterwards is not necessary and may cause a
    // needless overflow.
    if exp == 1 {
        acc = acc.wrapping_mul(base);
    }

    acc
}

// Borrowed from rosetta code
fn mod_inv(a: u64, module: u64) -> u64 {
    let mut mn = (module, a);
    let mut xy = (0u64, 1);

    while mn.1 != 0 {
        xy = (xy.1, xy.0.wrapping_sub((mn.0 / mn.1).wrapping_mul(xy.1)));
        mn = (mn.1, mn.0 % mn.1);
    }

    xy.0 % module
}

// Algorithm source:
// https://math.stackexchange.com/questions/2008585/computing-the-distance-between-two-linear-congruential-generator-states
fn distance_between_rngs(ss: &JavaRng, se: &JavaRng) -> u64 {
    let mut a = lcg_const::A;
    let mut c = lcg_const::C;
    let mut p = 1;
    let mut z = ss.get_raw_seed();
    let mut d = 0;

    while z != se.get_raw_seed() {
        if ((z ^ se.get_raw_seed()) & p) != 0 {
            z = a.wrapping_mul(z).wrapping_add(c) & mask(48);
            d += p;
        }

        c = c.wrapping_mul(a.wrapping_add(1));
        a = a.wrapping_mul(a);
        p <<= 1;
    }

    d
}

// Algorithm source:
// https://math.stackexchange.com/questions/2008585/computing-the-distance-between-two-linear-congruential-generator-states
fn distance_between_rngs_less_than(ss: &JavaRng, se: &JavaRng, limit: u64) -> Option<u64> {
    let mut a = lcg_const::A;
    let mut c = lcg_const::C;
    let mut p = 1;
    let mut z = ss.get_raw_seed();
    let mut d = 0;
    let mut i = 0;

    while z != se.get_raw_seed() {
        if d + p >= limit {
            return None;
        }
        if ((z ^ se.get_raw_seed()) & p) != 0 {
            d += p;
            if d >= limit {
                return None;
            }
            z = a.wrapping_mul(z).wrapping_add(c) & mask(48);
        }

        i += 1;
        c = c.wrapping_mul(a.wrapping_add(1));
        a = a.wrapping_mul(a);
        p <<= 1;
    }

    Some(d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_set_get() {
        let mut r = JavaRng { seed: 1234 };
        assert_eq!(r.get_seed(), 1234 ^ lcg_const::A);
        r.set_seed(lcg_const::A);
        assert_eq!(r.get_seed(), lcg_const::A);
        assert_eq!(r.seed, 0);
        r.set_seed(1 << 48); // 2^48 wraps back to 0
        assert_eq!(r.get_seed(), 0);
    }

    #[test]
    fn next_test() {
        let mut r = JavaRng::with_seed(12345);
        let i = r.next(32);
        assert_eq!(i, r.last_next(32));
    }

    #[test]
    fn previous() {
        let mut r = JavaRng::with_seed(12345);
        r.set_seed(12345);
        r.next_int();
        r.previous();
        assert_eq!(r.get_seed(), 12345);
    }

    #[test]
    fn long_from_ints() {
        let mut r = JavaRng::with_seed(12345);
        let l = r.next_long();
        r.set_seed(12345);
        let i0 = r.next_int();
        let i1 = r.next_int();
        let i = JavaRng::long_from_i0_i1(i0, i1);
        assert_eq!(l, i);
        let (j0, j1) = JavaRng::ints_from_long(l);
        assert_eq!(i0, j0);
        assert_eq!(i1, j1);
    }

    #[test]
    fn create_from_long() {
        let mut r = JavaRng::with_raw_seed(12345);
        let l = r.next_long();
        let rs = JavaRng::create_from_long(l as u64);
        assert_eq!(rs.unwrap().get_raw_seed(), 12345);
    }

    #[test]
    fn same_as_java() {
        let mut r = JavaRng::with_seed(12345);
        let l = r.next_long();
        assert_eq!(l, 6674089274190705457);
        r.set_seed(12345);
        let b = r.next_boolean();
        assert_eq!(b, false);
        r.set_seed(12345);
        let i = r.next_int_n(10);
        assert_eq!(i, 1);
        r.set_seed(12345);
        let i = r.next_int_n(16);
        assert_eq!(i, 5);
        r.set_seed(12345);
        let i = r.next_float();
        assert_eq!(i, 0.36180305);
        r.set_seed(12345);
        let i = r.next_double();
        // Interesting, nextDouble returns the same number as nextFloat
        assert_eq!(i, 0.3618031071604718);
    }

    #[test]
    fn next_double_is_same_as_next_float() {
        let max_diff = (1u64 << 29) as f64 / (1u64 << 53) as f64;
        for seed in 0..10 {
            let mut r1 = JavaRng::with_seed(seed);
            let mut r2 = r1.clone();

            r1.next_n_calls(1000);
            r2.next_n_calls(1000);

            let a = r1.next_double();
            let b = r2.next_float() as f64;
            let diff = (a - b).abs();

            assert!(diff < max_diff, "Seed {}: diff {}", seed, diff);
        }
    }

    #[test]
    fn approx_next_double() {
        let max_diff = (1u64 << 27) as f64 / (1u64 << 53) as f64;
        for seed in 0..10 {
            let mut r1 = JavaRng::with_seed(seed);
            let mut r2 = r1.clone();

            r1.next_n_calls(1000);
            r2.next_n_calls(1000);

            let a = r1.next_double();
            let b = ((r2.next(26) as i64) << 27) as f64 / (1u64 << 53) as f64;
            let diff = (a - b).abs();

            assert!(diff < max_diff, "Seed {}: diff {}", seed, diff);
        }
    }

    #[test]
    fn previous_calls_black_magic() {
        let mut r = JavaRng::with_seed(12345);
        r.previous();
        let c0 = r.get_seed();
        r.previous();
        let c1 = r.get_seed();
        r.set_seed(12345);
        r.previous_n_calls(1);
        let p0 = r.get_seed();
        r.set_seed(12345);
        r.previous_n_calls(2);
        let p1 = r.get_seed();

        assert_eq!(c0, p0);
        assert_eq!(c1, p1);

        r.set_seed(12345);
        for _ in 0..1000 {
            r.previous();
        }
        let p2 = r.get_seed();
        r.set_seed(12345);
        r.previous_n_calls(1000);
        let p3 = r.get_seed();
        assert_eq!(p2, p3);

        r.set_seed(12345);
        r.previous_n_calls(1 << 48);
        assert_eq!(r.get_seed(), 12345);
    }

    #[test]
    fn next_calls_black_magic() {
        let mut r = JavaRng::with_seed(12345);
        r.next_int();
        let c0 = r.get_seed();
        r.next_int();
        let c1 = r.get_seed();
        r.set_seed(12345);
        r.next_n_calls(1);
        let p0 = r.get_seed();
        r.set_seed(12345);
        r.next_n_calls(2);
        let p1 = r.get_seed();

        assert_eq!(c0, p0);
        assert_eq!(c1, p1);

        r.set_seed(12345);
        for _ in 0..1000 {
            r.next_int();
        }
        let p2 = r.get_seed();
        r.set_seed(12345);
        r.next_n_calls(1000);
        let p3 = r.get_seed();
        assert_eq!(p2, p3);

        r.set_seed(12345);
        r.next_n_calls(1 << 48);
        assert_eq!(r.get_seed(), 12345);
    }

    #[test]
    fn know_your_constants() {
        assert_eq!(mod_inv(lcg_const::A, 1 << 48), lcg_const_extra::INV_A);
        assert_eq!(
            mod_inv((lcg_const_extra::INV_A - 1) >> 2, 1 << 48),
            lcg_const_extra::INV__INV_A__1
        );
        assert_eq!(
            mod_inv((lcg_const::A - 1) >> 2, 1 << 48),
            lcg_const_extra::INV_A_1
        );
        assert_eq!(2147483640, (1u32 << 31) / 10 * 10);
    }

    #[test]
    fn modulo_bias_next_int_n() {
        let mut r = JavaRng::with_seed(12345678);
        let x = r.next_int_n((1 << 30) + 1);
        assert_eq!(x, 677997345);
    }

    #[test]
    fn next_int_n_10() {
        let s = 1_356_836_617;
        let mut r0 = JavaRng::with_seed(s);
        let mut r1 = r0.clone();
        let x0 = r0.next_int_n(10);
        let x1 = r1.next_int_n_10();
        assert_eq!(x0, 6);
        assert_eq!(x0, x1);

        for target in 2147483630..2147483648 {
            let mut rt = JavaRng::with_raw_seed(target << 17);
            rt.previous();
            let mut rtc = rt.clone();
            assert_eq!((target, rt.next_int_n(10)), (target, rtc.next_int_n_10()));
        }
    }

    #[test]
    fn extend_48_to_64() {
        assert_eq!(
            &JavaRng::extend_long_48(132607203138509),
            &[4400149443144113101]
        );
        assert_eq!(
            &JavaRng::extend_long_48(113453751637441),
            &[6895687433209288129, 955720999684314561]
        );
        assert_eq!(
            &JavaRng::extend_long_48(18021957452394),
            &[3640896845709787754]
        );
        assert_eq!(
            &JavaRng::extend_long_48(131291916928825),
            &[-1095369317440944327i64 as u64]
        );
        assert_eq!(
            &JavaRng::extend_long_48(249127199878301),
            &[5773582374512143517, -166384059012830051i64 as u64]
        );
        assert_eq!(
            &JavaRng::extend_long_48(186701866325681),
            &[3353398099420370609, -2586568334104602959i64 as u64]
        );
    }

    #[test]
    fn test_low_16() {
        let s = 1234;
        let mut r = JavaRng::with_seed(s);

        for _ in 0..1 {
            let i0 = r.next_int();
            let expected = (r.get_raw_seed() & mask(16)) as u16;
            let i1 = r.next_int();
            let ee = JavaRng::low_16_for_next_int(i0 as u32, i1 as u32);
            assert_eq!(ee, Some(expected));
        }
    }

    #[test]
    fn distance_fn0() {
        let r = JavaRng::with_seed(12345);
        assert_eq!(distance_between_rngs(&r, &r), 0);
    }

    #[test]
    fn distance_fn1() {
        let r0 = JavaRng::with_seed(12345);
        let mut r = r0;
        r.next(32);
        assert_eq!(distance_between_rngs(&r0, &r), 1);
    }

    #[test]
    fn distance_fn2() {
        let r0 = JavaRng::with_seed(12345);
        let mut r = r0;
        r.next(32);
        r.next(32);
        assert_eq!(distance_between_rngs(&r0, &r), 2);
    }

    #[test]
    fn distance_fn99() {
        let r0 = JavaRng::with_seed(12345);
        let mut r = r0;
        for _ in 0..99 {
            r.next(32);
        }
        assert_eq!(distance_between_rngs(&r0, &r), 99);
    }

    #[test]
    fn distance_limit_fn99() {
        let r0 = JavaRng::with_seed(12345);
        let mut r = r0;
        for _ in 0..99 {
            r.next(32);
        }
        assert_eq!(distance_between_rngs_less_than(&r0, &r, 100), Some(99));
        assert_eq!(distance_between_rngs_less_than(&r0, &r, 99), None);
        assert_eq!(distance_between_rngs_less_than(&r0, &r, 98), None);
    }
}
