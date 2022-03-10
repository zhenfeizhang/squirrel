use crate::param::SMALL_MODULUS_BITS;
use crate::poly::{SmallNTTPoly, SmallPoly};
use crate::SignedPoly;
use rand::Rng;
#[cfg(feature = "parallel")]
use rayon::iter::IntoParallelIterator;
#[cfg(feature = "parallel")]
use rayon::iter::ParallelIterator;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct HVCHash {
    pub(crate) param_h: [SmallNTTPoly; 28],
}

impl HVCHash {
    pub fn init<R: Rng>(rng: &mut R) -> Self {
        let mut res = Self::default();

        for e in res.param_h.iter_mut() {
            let tmp = SmallPoly::rand_poly(rng);
            *e = (&tmp).into();
        }
        res
    }

    /// Hash function.
    /// Cost: 2*SMALL_MODULUS_BITS NTT and 1 INV_NTT.
    pub fn hash(&self, inputs: &[SmallPoly]) -> SmallPoly {
        // TODO: check the cost for fixed bases
        // may be faster than NTT

        assert_eq!(inputs.len(), SMALL_MODULUS_BITS << 1);

        let mut res = SmallNTTPoly::default();

        #[cfg(feature = "parallel")]
        let prod_ntt: Vec<SmallNTTPoly> = self
            .param_h
            .iter()
            .zip(inputs.iter())
            .map(|(x, y)| (*x, *y))
            .collect::<Vec<(SmallNTTPoly, SmallPoly)>>()
            .into_par_iter()
            .map(|(x, y)| x * SmallNTTPoly::from(&y))
            .collect();

        #[cfg(not(feature = "parallel"))]
        let prod_ntt: Vec<SmallNTTPoly> = self
            .param_h
            .iter()
            .zip(inputs.iter())
            .map(|(x, y)| *x * SmallNTTPoly::from(y))
            .collect();

        for e in prod_ntt {
            res += e;
        }

        // convert the polynomial from NTT domain back to integers
        (&res).into()
    }

    pub(crate) fn decom_then_hash(&self, left: &SmallPoly, right: &SmallPoly) -> SmallPoly {
        self.hash_separate_inputs(&left.decompose(), &right.decompose())
    }

    pub(crate) fn hash_separate_inputs(
        &self,
        left: &[SignedPoly],
        right: &[SignedPoly],
    ) -> SmallPoly {
        let inputs: Vec<SmallPoly> = [left, right]
            .concat()
            .iter()
            .map(|&x| x.lifted_small())
            .collect();
        self.hash(&inputs)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::poly::TerPolyCoeffEncoding;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn test_hash() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        let hasher = HVCHash::init(&mut rng);

        let inputs: Vec<SmallPoly> = (0..SMALL_MODULUS_BITS << 1)
            .map(|_| SmallPoly::rand_poly(&mut rng))
            .collect();
        let _ = hasher.hash(&inputs);
    }

    #[test]
    fn test_homomorphism() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
        let hasher = HVCHash::init(&mut rng);

        for _ in 0..10 {
            {
                let poly1 = SmallPoly::rand_poly(&mut rng);
                let decomposed_poly1 = poly1.decompose();
                let poly2 = SmallPoly::rand_poly(&mut rng);
                let decomposed_poly2 = poly2.decompose();
                let decomposed: Vec<SignedPoly> = decomposed_poly1
                    .iter()
                    .zip(decomposed_poly2.iter())
                    .map(|(&x, &y)| x + y)
                    .collect();
                let poly_rec = SmallPoly::projection(&decomposed);
                let poly = poly1 + poly2;
                assert_eq!(poly, poly_rec);
            }
            {
                let poly1 = SmallPoly::rand_poly(&mut rng);
                let decomposed_poly1 = poly1.decompose();
                let poly2 = SmallPoly::rand_poly(&mut rng);
                let decomposed_poly2 = poly2.decompose();
                let r1 = SignedPoly::rand_ternary(&mut rng, 11);
                let r2 = SignedPoly::rand_ternary(&mut rng, 11);
                let randomizer1: TerPolyCoeffEncoding = (&r1).into();
                let randomizer2: TerPolyCoeffEncoding = (&r2).into();
                let decomposed_poly1: Vec<SignedPoly> = decomposed_poly1
                    .iter()
                    .map(|&x| SignedPoly::ter_mul_bin(&randomizer1, &x))
                    .collect();
                let decomposed_poly2: Vec<SignedPoly> = decomposed_poly2
                    .iter()
                    .map(|&x| SignedPoly::ter_mul_bin(&randomizer2, &x))
                    .collect();
                let decomposed: Vec<SignedPoly> = decomposed_poly1
                    .iter()
                    .zip(decomposed_poly2.iter())
                    .map(|(&x, &y)| x + y)
                    .collect();
                let poly_rec = SmallPoly::projection(&decomposed);
                let poly = poly1 * (&r1).into() + poly2 * (&r2).into();
                assert_eq!(poly, poly_rec);
            }
            {
                let r1 = SignedPoly::rand_ternary(&mut rng, 11);
                let r2 = SignedPoly::rand_ternary(&mut rng, 11);
                let randomizer1: TerPolyCoeffEncoding = (&r1).into();
                let randomizer2: TerPolyCoeffEncoding = (&r2).into();

                let poly11 = SmallPoly::rand_poly(&mut rng);
                let poly12 = SmallPoly::rand_poly(&mut rng);
                let poly11_randomized: Vec<SignedPoly> = poly11
                    .decompose()
                    .iter()
                    .map(|&x| SignedPoly::ter_mul_bin(&randomizer1, &x))
                    .collect();
                let poly12_randomized: Vec<SignedPoly> = poly12
                    .decompose()
                    .iter()
                    .map(|&x| SignedPoly::ter_mul_bin(&randomizer1, &x))
                    .collect();

                let poly21 = SmallPoly::rand_poly(&mut rng);
                let poly22 = SmallPoly::rand_poly(&mut rng);
                let poly21_randomized: Vec<SignedPoly> = poly21
                    .decompose()
                    .iter()
                    .map(|&x| SignedPoly::ter_mul_bin(&randomizer2, &x))
                    .collect();
                let poly22_randomized: Vec<SignedPoly> = poly22
                    .decompose()
                    .iter()
                    .map(|&x| SignedPoly::ter_mul_bin(&randomizer2, &x))
                    .collect();

                let poly1 = hasher.decom_then_hash(&poly11, &poly12);
                let poly2 = hasher.decom_then_hash(&poly21, &poly22);
                let poly = poly1 * (&r1).into() + poly2 * (&r2).into();

                let polyx1_randomized: Vec<SignedPoly> = poly11_randomized
                    .iter()
                    .zip(poly21_randomized.iter())
                    .map(|(&x, &y)| x + y)
                    .collect();
                let polyx2_randomized: Vec<SignedPoly> = poly12_randomized
                    .iter()
                    .zip(poly22_randomized.iter())
                    .map(|(&x, &y)| x + y)
                    .collect();
                let poly_rec = hasher.hash_separate_inputs(&polyx1_randomized, &polyx2_randomized);

                assert_eq!(poly, poly_rec);
            }
        }
    }
}
