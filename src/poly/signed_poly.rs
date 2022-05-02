use super::LargePoly;
use super::SmallPoly;
use super::TerPolyCoeffEncoding;
use crate::SignedPoly;
use crate::ALPHA;
use crate::BETA_S;
use crate::BETA_S_SAMPLE_THRESHOLD;
use crate::LARGE_MODULUS;
use crate::N;
use crate::SMALL_MODULUS;
use crate::TWO_BETA_S_PLUS_ONE;
use core::fmt;
use cpoly::ternary_mul;
use rand::Rng;
use rand::RngCore;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use sha2::Digest;
use sha2::Sha256;
use std::fmt::Display;
use std::ops::Add;
use std::ops::AddAssign;

impl Default for SignedPoly {
    fn default() -> Self {
        Self { coeffs: [0i32; N] }
    }
}

impl Display for SignedPoly {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#06x} {:#06x}", self.coeffs[0], self.coeffs[1])
    }
}

impl Add for SignedPoly {
    type Output = Self;

    // Coefficient wise additions without mod reduction.
    fn add(self, other: Self) -> Self {
        let mut res = Self::default();

        #[cfg(debug_assertions)]
        for (e, (f, g)) in res
            .coeffs
            .iter_mut()
            .zip(self.coeffs.iter().zip(other.coeffs))
        {
            *e = match i32::checked_add(*f, g) {
                Some(p) => p,
                None => {
                    panic!("overflowing additions")
                }
            };
        }

        #[cfg(not(debug_assertions))]
        for (e, (f, g)) in res
            .coeffs
            .iter_mut()
            .zip(self.coeffs.iter().zip(other.coeffs.iter()))
        {
            *e = f + g;
        }
        res
    }
}

impl AddAssign for SignedPoly {
    // Coefficient wise additions without mod reduction.
    fn add_assign(&mut self, other: Self) {
        #[cfg(debug_assertions)]
        for (x, y) in self.coeffs.iter_mut().zip(other.coeffs) {
            *x = match i32::checked_add(*x, y) {
                Some(p) => p,
                None => {
                    panic!("overflowing additions")
                }
            };
        }
        #[cfg(not(debug_assertions))]
        for (x, y) in self.coeffs.iter_mut().zip(other.coeffs) {
            *x = *x + y
        }
    }
}

impl SignedPoly {
    // school book multiplication
    // slow. only used for correctness checking
    #[cfg(test)]
    pub(crate) fn schoolbook(a: &Self, b: &Self, q: i32) -> Self {
        let mut buf = [0i64; N * 2];
        let mut c = [0; N];
        for i in 0..N {
            for j in 0..N {
                buf[i + j] += a.coeffs[i] as i64 * b.coeffs[j] as i64 % q as i64;
            }
        }
        for i in 0..N {
            c[i] = ((buf[i] - buf[i + N]) % q as i64) as i32;
        }
        Self { coeffs: c }
    }

    // multiply a ternary with a binary poly
    pub fn ter_mul_bin(ter: &TerPolyCoeffEncoding, bin: &Self) -> Self {
        #[cfg(debug_assertions)]
        assert!(bin.is_binary());

        let mut res = Self::default();
        let mut tmp = [0i8; N];
        let mut buf = [0u8; 2 * N];
        let bin: Vec<i8> = bin.coeffs.iter().map(|&x| x as i8).collect();
        let ter: Vec<u8> = ter.indices.iter().map(|&x| x as u8).collect();

        unsafe {
            ternary_mul(
                tmp.as_mut_ptr(),
                buf.as_mut_ptr(),
                bin.as_ptr(),
                ter.as_ptr(),
            );
        }
        for (e, f) in res.coeffs.iter_mut().zip(tmp.iter()) {
            *e = *f as i32
        }
        res
    }

    // sample a random ternary polynomial with a fixed weight
    pub fn rand_ternary<R: Rng>(rng: &mut R, half_weight: usize) -> Self {
        let mut ct = 0;
        let mut coeffs = [0; N];
        let mut rng_ct = 0;
        let mut tmp = rng.next_u32();

        while ct < half_weight {
            let index = (tmp & 0xFF) as usize;
            tmp >>= 9;
            rng_ct += 1;
            if rng_ct == 3 {
                tmp = rng.next_u32();
                rng_ct = 0;
            }
            if coeffs[index] == 0 {
                ct += 1;
                coeffs[index] = 1
            }
        }
        ct = 0;
        while ct < half_weight {
            let index = (tmp & 0xFF) as usize;
            tmp >>= 9;
            rng_ct += 1;
            if rng_ct == 3 {
                tmp = rng.next_u32();
                rng_ct = 0;
            }

            if coeffs[index] == 0 {
                ct += 1;
                coeffs[index] = -1
            }
        }
        Self { coeffs }
    }

    pub(crate) fn is_ternary(&self) -> bool {
        for &e in self.coeffs.iter() {
            if e != 0 && e != 1 && e != -1 {
                return false;
            }
        }
        true
    }

    pub(crate) fn is_binary(&self) -> bool {
        for &e in self.coeffs.iter() {
            if e != 0 && e != 1 {
                return false;
            }
        }
        true
    }

    /// sample a random polynomial with coefficients between [-beta_s, beta_s]
    pub fn rand_mod_beta_s<R: Rng>(rng: &mut R) -> Self {
        // todo: improve sampling rates
        let mut res = Self::default();
        for e in res.coeffs.iter_mut() {
            let mut tmp = rng.next_u32();
            while tmp > BETA_S_SAMPLE_THRESHOLD {
                tmp = rng.next_u32();
            }

            *e = (tmp % TWO_BETA_S_PLUS_ONE) as i32 - BETA_S as i32;
        }

        res
    }

    // sample a random binary polynomial
    pub fn rand_binary<R: Rng>(rng: &mut R) -> Self {
        let mut res = Self::default();
        for i in 0..16 {
            let mut tmp = rng.next_u32();
            for j in 0..32 {
                res.coeffs[i * 32 + j] = (tmp & 1) as i32;
                tmp >>= 1;
            }
        }

        res
    }

    // sample a random binary polynomial with a fixed weight
    pub fn rand_fixed_weight_binary<R: Rng>(rng: &mut R, weight: usize) -> Self {
        let mut ct = 0;
        let mut coeffs = [0; N];
        let mut tmp_ct = 0;
        let mut tmp = rng.next_u32();
        while ct < weight {
            let index = (tmp & 0xFF) as usize;
            tmp >>= 9;
            tmp_ct += 1;
            if tmp_ct == 3 {
                tmp = rng.next_u32();
                tmp_ct = 0;
            }

            if coeffs[index] == 0 {
                ct += 1;
                coeffs[index] = 1
            }
        }

        Self { coeffs }
    }

    // sample a random binary polynomial with a fixed weight
    pub fn rand_fixed_weight_ternary<R: Rng>(rng: &mut R, weight: usize) -> Self {
        #[cfg(debug_assertions)]
        assert!(weight <= 64);

        let mut ct = 0;
        let mut coeffs = [0; N];
        let mut tmp_ct = 0;
        let mut tmp = rng.next_u32();
        let mut sign = rng.next_u64();

        while ct < weight {
            let index = (tmp & 0xFF) as usize;
            tmp >>= 9;
            tmp_ct += 1;
            if tmp_ct == 3 {
                tmp = rng.next_u32();
                tmp_ct = 0;
            }

            if coeffs[index] == 0 {
                ct += 1;
                if sign & 1 == 0 {
                    coeffs[index] = 1
                } else {
                    coeffs[index] = -1
                }
                sign >>= 1;
            }
        }

        Self { coeffs }
    }

    pub(crate) fn lifted_small(&self) -> SmallPoly {
        let mut res = *self;
        for e in res.coeffs.iter_mut() {
            *e = (*e % SMALL_MODULUS as i32 + SMALL_MODULUS as i32) % SMALL_MODULUS as i32
        }
        (&res).into()
    }

    pub(crate) fn lifted_large(&self) -> LargePoly {
        let mut res = *self;
        res.coeffs.iter_mut().for_each(|e| {
            *e = (*e % LARGE_MODULUS as i32 + LARGE_MODULUS as i32) % LARGE_MODULUS as i32
        });
        (&res).into()
    }

    /// hash a blob into a message polynomial
    pub(crate) fn from_hash_message(msg: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(msg);
        let seed = hasher.finalize().into();
        let mut rng = ChaCha20Rng::from_seed(seed);

        let mut res = Self::default();
        let mut ct = 0;
        // todo: improve sampling rates
        while ct < BETA_S {
            let tmp = rng.next_u32();
            if res.coeffs[(tmp % N as u32) as usize] == 0 {
                ct += 1;
                if (tmp >> 9) & 1 == 1 {
                    res.coeffs[(tmp % N as u32) as usize] = 1;
                } else {
                    res.coeffs[(tmp % N as u32) as usize] = -1;
                }
            }
        }
        res
    }
}

impl From<&SignedPoly> for TerPolyCoeffEncoding {
    fn from(poly: &SignedPoly) -> Self {
        // TODO: this conversion should only be possible if poly has same number of
        // +/- 1s. Add a check for this.

        #[cfg(debug_assertions)]
        assert!(poly.is_ternary());

        let mut indices = [0usize; ALPHA];
        let mut ct = 0;
        for (index, &coeff) in poly.coeffs.iter().enumerate() {
            if coeff == 1 {
                indices[ct] = index;
                ct += 1;
            }
        }
        for (index, &coeff) in poly.coeffs.iter().enumerate() {
            if coeff == -1 {
                indices[ct] = index;
                ct += 1;
            }
        }

        Self { indices }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::poly::LargePoly;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;
    #[test]
    fn test_ter_mul() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
        let half_weight = 10;

        for _ in 0..10 {
            let ter_poly = SignedPoly::rand_ternary(&mut rng, half_weight);
            let bin_poly = SignedPoly::rand_binary(&mut rng);
            let ter_poly_coeff_encoding: TerPolyCoeffEncoding = (&ter_poly).into();

            let prod_1 = SignedPoly::schoolbook(&bin_poly, &ter_poly, SMALL_MODULUS as i32);
            let prod_2 = SignedPoly::ter_mul_bin(&ter_poly_coeff_encoding, &bin_poly);
            let prod_3 = SmallPoly::from(&ter_poly) * SmallPoly::from(&bin_poly);
            let prod_4 = LargePoly::from(&ter_poly) * LargePoly::from(&bin_poly);
            assert_eq!(prod_1.lifted_small(), prod_2.lifted_small());
            assert_eq!(prod_1.lifted_small(), prod_3);
            assert_eq!(prod_1.lifted_large(), prod_4)
        }
    }
}
