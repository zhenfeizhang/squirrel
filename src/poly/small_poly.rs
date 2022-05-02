use cpoly::hvc_inv_ntt;
use cpoly::hvc_ntt;
use rand::Rng;
use sha2::Digest;
use sha2::Sha256;

use super::SignedPoly;
use super::SmallNTTPoly;
use super::SmallPoly;
use crate::N;
use crate::SMALL_MODULUS as MODULUS;
use crate::SMALL_MODULUS_BITS;
use crate::SMALL_SAMPLE_THRESHOLD;
use core::fmt;
use std::fmt::Display;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Mul;

impl Default for SmallPoly {
    fn default() -> Self {
        Self { coeffs: [0u16; N] }
    }
}

impl Display for SmallPoly {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#06x} {:#06x}", self.coeffs[0], self.coeffs[1])
    }
}

impl From<&SmallPoly> for SignedPoly {
    fn from(input: &SmallPoly) -> Self {
        let mut res = Self::default();
        res.coeffs
            .iter_mut()
            .zip(input.coeffs)
            .for_each(|(e, f)| *e = f as i32);

        res
    }
}

impl From<&SignedPoly> for SmallPoly {
    fn from(input: &SignedPoly) -> Self {
        let mut res = Self::default();
        res.coeffs
            .iter_mut()
            .zip(input.coeffs)
            .for_each(|(e, f)| *e = lift(f) as u16);

        res
    }
}

impl Add for SmallPoly {
    type Output = Self;

    // Coefficient wise additions without mod reduction.
    fn add(self, other: Self) -> Self {
        let mut res = self;
        res += other;
        res
    }
}

impl AddAssign for SmallPoly {
    // Coefficient wise additions with mod reduction.
    fn add_assign(&mut self, other: Self) {
        self.coeffs
            .iter_mut()
            .zip(other.coeffs)
            .for_each(|(x, y)| *x = ((*x as u32 + y as u32) % MODULUS as u32) as u16)
    }
}

impl Mul for SmallPoly {
    type Output = Self;

    // Ring multiplication
    fn mul(self, other: Self) -> Self {
        (&(SmallNTTPoly::from(&self) * SmallNTTPoly::from(&other))).into()
    }
}

impl SmallPoly {
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
            c[i] = lift(((buf[i] - buf[i + N]) % MODULUS as i64) as i32);
        }
        Self { coeffs: c }
    }

    /// sample a random polynomial with coefficients between 0 and q-1
    /// should only be used for testing or so
    #[cfg(test)]
    pub fn rand_non_uniform_poly<R: Rng>(rng: &mut R) -> Self {
        let mut res = Self::default();
        for e in res.coeffs.iter_mut() {
            *e = rng.next_u32() as u16 % MODULUS
        }
        res
    }

    /// sample a uniformly random polynomial with coefficients between 0 and q-1
    pub fn rand_poly<R: Rng>(rng: &mut R) -> Self {
        let mut res = Self::default();
        for e in res.coeffs.iter_mut() {
            let mut tmp = rng.next_u32();
            while tmp >= SMALL_SAMPLE_THRESHOLD {
                tmp = rng.next_u32();
            }
            *e = (tmp % MODULUS as u32) as u16
        }
        res
    }

    /// decompose a mod q polynomial into binary polynomials
    pub fn decompose(&self) -> [SignedPoly; SMALL_MODULUS_BITS] {
        let mut res = [SignedPoly::default(); SMALL_MODULUS_BITS];
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
        let mut res = binary_polys[SMALL_MODULUS_BITS - 1];
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

impl Default for SmallNTTPoly {
    fn default() -> Self {
        Self { coeffs: [0u16; N] }
    }
}

impl From<&SignedPoly> for SmallNTTPoly {
    // convert poly into its ntt form. Requires that coefficients are between 0 and 61441
    fn from(poly: &SignedPoly) -> Self {
        (&SmallPoly::from(poly)).into()
    }
}

impl From<&SmallPoly> for SmallNTTPoly {
    // convert poly into its ntt form. Requires that coefficients are between 0 and 61441
    fn from(poly: &SmallPoly) -> Self {
        let mut coeffs = poly.coeffs;
        unsafe {
            hvc_ntt(coeffs.as_mut_ptr());
        }
        Self { coeffs }
    }
}

impl From<&SmallNTTPoly> for SmallPoly {
    fn from(poly: &SmallNTTPoly) -> Self {
        let mut coeffs = poly.coeffs;
        unsafe {
            hvc_inv_ntt(coeffs.as_mut_ptr());
        }
        Self { coeffs }
    }
}

impl Add for SmallNTTPoly {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut res = Self::default();
        for (e, (f, g)) in res
            .coeffs
            .iter_mut()
            .zip(self.coeffs.iter().zip(other.coeffs.iter()))
        {
            *e = ((*f as u32 + *g as u32) % MODULUS as u32) as u16
        }

        res
    }
}

impl AddAssign for SmallNTTPoly {
    fn add_assign(&mut self, other: SmallNTTPoly) {
        for (x, y) in self.coeffs.iter_mut().zip(other.coeffs) {
            *x = ((*x as u32 + y as u32) % MODULUS as u32) as u16
        }
    }
}

impl Mul for SmallNTTPoly {
    type Output = Self;

    // Coefficient-wise multiplication over the NTT domain.
    fn mul(self, other: Self) -> Self {
        let mut res = Self::default();
        for (e, (f, g)) in res
            .coeffs
            .iter_mut()
            .zip(self.coeffs.iter().zip(other.coeffs.iter()))
        {
            *e = (((*f as u32) * (*g as u32)) % MODULUS as u32) as u16
        }

        res
    }
}

fn lift(a: i32) -> u16 {
    ((a % MODULUS as i32 + MODULUS as i32) % MODULUS as i32) as u16
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
            let poly = SmallPoly::rand_poly(&mut rng);
            let poly_ntt: SmallNTTPoly = (&poly).into();
            let poly_rec: SmallPoly = (&poly_ntt).into();

            assert_eq!(poly, poly_rec)
        }
    }

    #[test]
    fn test_arithmetic() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
        for _ in 0..10 {
            let a = SmallPoly::rand_poly(&mut rng);
            let a_ntt: SmallNTTPoly = (&a).into();
            let b = SmallPoly::rand_poly(&mut rng);
            let b_ntt: SmallNTTPoly = (&b).into();

            {
                // test correctness of ntt multiplications
                let c_ntt = a_ntt * b_ntt;
                let c: SmallPoly = (&c_ntt).into();
                let c_rec = SmallPoly::schoolbook(&a, &b);

                assert_eq!(c, c_rec);
            }
            {
                // test correctness of ntt additions
                let d_ntt = a_ntt + b_ntt;
                let d: SmallPoly = (&d_ntt).into();
                let d_rec = a + b;

                assert_eq!(d, d_rec)
            }
        }
    }

    #[test]
    fn test_decomposition() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        for _ in 0..10 {
            let poly = SmallPoly::rand_poly(&mut rng);
            let decomposed = poly.decompose();
            let poly_rec = SmallPoly::projection(&decomposed);
            assert_eq!(poly, poly_rec);
        }
    }
}
