use super::LargeNTTPoly;
use super::LargePoly;
use super::SignedPoly;
use crate::LARGE_MODULUS as MODULUS;
use crate::LARGE_MODULUS_BITS;
use crate::LARGE_SAMPLE_THRESHOLD;
use crate::N;
use core::fmt;
use cpoly::hots_inv_ntt;
use cpoly::hots_ntt;
use rand::Rng;
use sha2::Digest;
use sha2::Sha256;
use std::fmt::Display;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Mul;

impl Default for LargePoly {
    fn default() -> Self {
        Self { coeffs: [0u32; N] }
    }
}

impl Display for LargePoly {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#06x} {:#06x}", self.coeffs[0], self.coeffs[1])
    }
}

impl From<&LargePoly> for SignedPoly {
    fn from(input: &LargePoly) -> Self {
        let mut res = Self::default();

        res.coeffs
            .iter_mut()
            .zip(input.coeffs)
            .for_each(|(e, f)| *e = f as i32);

        res
    }
}

impl From<&SignedPoly> for LargePoly {
    fn from(input: &SignedPoly) -> Self {
        let mut res = Self::default();
        res.coeffs
            .iter_mut()
            .zip(input.coeffs)
            .for_each(|(e, f)| *e = lift(f as i64));

        res
    }
}

impl Add for LargePoly {
    type Output = Self;

    // Coefficient wise additions without mod reduction.
    fn add(self, other: Self) -> Self {
        let mut res = self;
        res += other;
        res
    }
}

impl AddAssign for LargePoly {
    // Coefficient wise additions with mod reduction.
    fn add_assign(&mut self, other: Self) {
        self.coeffs
            .iter_mut()
            .zip(other.coeffs)
            .for_each(|(x, y)| *x = (*x + y) % MODULUS)
    }
}

impl Mul for LargePoly {
    type Output = Self;

    // Ring multiplication
    fn mul(self, other: Self) -> Self {
        (&(LargeNTTPoly::from(&self) * LargeNTTPoly::from(&other))).into()
    }
}

impl LargePoly {
    // school book multiplication
    // slow. only used for correctness checking
    #[cfg(test)]
    pub(crate) fn schoolbook(a: &Self, b: &Self) -> Self {
        let mut buf = [0i64; N * 2];
        let mut c = [0; N];
        for i in 0..N {
            for j in 0..N {
                buf[i + j] += (a.coeffs[i] as i64) * (b.coeffs[j] as i64) % (MODULUS as i64);
            }
        }
        for i in 0..N {
            c[i] = lift(buf[i] - buf[i + N]);
        }
        Self { coeffs: c }
    }

    /// sample a random polynomial with coefficients between 0 and q-1
    /// should only be used for testing or so
    #[cfg(test)]
    pub fn rand_non_uniform_poly<R: Rng>(rng: &mut R) -> Self {
        let mut res = Self::default();
        for e in res.coeffs.iter_mut() {
            *e = rng.next_u32() % MODULUS
        }
        res
    }

    /// sample a random polynomial with coefficients between 0 and q-1
    pub fn rand_poly<R: Rng>(rng: &mut R) -> Self {
        let mut res = Self::default();
        for e in res.coeffs.iter_mut() {
            let mut tmp = rng.next_u32();
            while tmp >= LARGE_SAMPLE_THRESHOLD {
                tmp = rng.next_u32();
            }
            *e = tmp % MODULUS
        }
        res
    }

    /// decompose a mod q polynomial into binary polynomials
    pub fn decompose(&self) -> [SignedPoly; LARGE_MODULUS_BITS] {
        let mut res = [SignedPoly::default(); LARGE_MODULUS_BITS];
        let mut base_coeffs = self.coeffs;
        for poly in res.iter_mut() {
            for (tar_coeff, cur_coeff) in (*poly).coeffs.iter_mut().zip(base_coeffs.iter_mut()) {
                *tar_coeff = ((*cur_coeff) & 1) as i32;
                (*cur_coeff) >>= 1;
            }
        }

        res
    }

    /// project a set of vectors to R_q
    pub fn projection(binary_polys: &[SignedPoly]) -> Self {
        let mut res = binary_polys[LARGE_MODULUS_BITS - 1];
        for binary_poly in binary_polys.iter().rev().skip(1) {
            for (res, &base) in res.coeffs.iter_mut().zip(binary_poly.coeffs.iter()) {
                *res <<= 1;
                *res += base;
            }
        }
        (&res).into()
    }

    /// A 256 digest of the polynomial
    pub(crate) fn digest(&self) -> [u8; 32] {
        let mut inputs = Vec::new();
        for e in self.coeffs {
            inputs.push((e & 0xFF) as u8);
            inputs.push(((e >> 8) & 0xFF) as u8);
        }
        let mut hasher = Sha256::new();
        hasher.update(inputs);
        let result = hasher.finalize();
        result.into()
    }
}

impl Default for LargeNTTPoly {
    fn default() -> Self {
        Self { coeffs: [0u32; N] }
    }
}

impl From<&SignedPoly> for LargeNTTPoly {
    // convert poly into its ntt form. Requires that coefficients are between 0 and 61441
    fn from(poly: &SignedPoly) -> Self {
        (&LargePoly::from(poly)).into()
    }
}

impl From<&LargePoly> for LargeNTTPoly {
    // convert poly into its ntt form. Requires that coefficients are between 0 and 61441
    fn from(poly: &LargePoly) -> Self {
        let mut coeffs = poly.coeffs;
        unsafe {
            hots_ntt(coeffs.as_mut_ptr());
        }
        Self { coeffs }
    }
}

impl From<&LargeNTTPoly> for LargePoly {
    fn from(poly: &LargeNTTPoly) -> Self {
        let mut coeffs = poly.coeffs;
        unsafe {
            hots_inv_ntt(coeffs.as_mut_ptr());
        }
        Self { coeffs }
    }
}

impl Add for LargeNTTPoly {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut res = Self::default();
        for (e, (f, g)) in res
            .coeffs
            .iter_mut()
            .zip(self.coeffs.iter().zip(other.coeffs.iter()))
        {
            *e = (f + g) % MODULUS
        }

        res
    }
}

impl AddAssign for LargeNTTPoly {
    fn add_assign(&mut self, other: LargeNTTPoly) {
        for (x, y) in self.coeffs.iter_mut().zip(other.coeffs) {
            *x = (*x + y) % MODULUS
        }
    }
}

impl Mul for LargeNTTPoly {
    type Output = Self;

    // Coefficient-wise multiplication over the NTT domain.
    fn mul(self, other: Self) -> Self {
        let mut res = Self::default();
        for (e, (f, g)) in res
            .coeffs
            .iter_mut()
            .zip(self.coeffs.iter().zip(other.coeffs.iter()))
        {
            *e = (((*f as u64) * (*g as u64)) % MODULUS as u64) as u32
        }

        res
    }
}

// #[inline]
fn lift(a: i64) -> u32 {
    (a % MODULUS as i64 + MODULUS as i64) as u32 % MODULUS
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn test_conversion() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
        for _ in 0..10 {
            let poly = LargePoly::rand_poly(&mut rng);
            let poly_ntt: LargeNTTPoly = (&poly).into();
            let poly_rec: LargePoly = (&poly_ntt).into();

            assert_eq!(poly, poly_rec)
        }
    }

    #[test]
    fn test_arithmetic() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
        for _ in 0..10 {
            let a = LargePoly::rand_poly(&mut rng);
            let a_ntt: LargeNTTPoly = (&a).into();
            let b = LargePoly::rand_poly(&mut rng);
            let b_ntt: LargeNTTPoly = (&b).into();

            {
                // test correctness of ntt multiplications
                let c_ntt = a_ntt * b_ntt;
                let c: LargePoly = (&c_ntt).into();
                let c_rec = LargePoly::schoolbook(&a, &b);

                assert_eq!(c, c_rec);
            }
            {
                // test correctness of ntt additions
                let d_ntt = a_ntt + b_ntt;
                let d: LargePoly = (&d_ntt).into();
                let d_rec = a + b;

                assert_eq!(d, d_rec)
            }
        }
    }

    #[test]
    fn test_decomposition() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        for _ in 0..10 {
            let poly = LargePoly::rand_poly(&mut rng);
            let decomposed = poly.decompose();
            let poly_rec = LargePoly::projection(&decomposed);
            assert_eq!(poly, poly_rec);
        }
    }
}
