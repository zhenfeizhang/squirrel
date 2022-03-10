use std::ops::AddAssign;

use crate::{
    poly::{LargePoly, SmallPoly, TerPolyCoeffEncoding},
    HOTSHash, Randomizers, SignedPoly, LARGE_MODULUS_BITS,
};
#[cfg(feature = "parallel")]
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

// HOTS public key
#[derive(Debug, Default, Clone, Copy)]
pub struct HotsPK {
    pub(crate) v0: LargePoly,
    pub(crate) v1: LargePoly,
}

impl HotsPK {
    pub(crate) fn digest(&self, hasher: &HOTSHash) -> SmallPoly {
        hasher.hash_separate_inputs(&self.v0.decompose(), &self.v1.decompose())
    }

    /// Aggregate multiple PKs into a single PK
    pub(crate) fn aggregate(pks: &[Self], roots: &[SmallPoly]) -> Self {
        // get and apply the randomizers
        let randomizers = Randomizers::from_pks(roots);
        Self::aggregate_with_randomizers(pks, &randomizers)
    }

    /// Aggregate a set of pks with randomizes
    pub(crate) fn aggregate_with_randomizers(pks: &[Self], randomizers: &Randomizers) -> Self {
        let mut pk_and_randomizer: Vec<(Self, LargePoly)> = pks
            .iter()
            .zip(randomizers.poly.iter())
            .map(|(&pk, r)| (pk, LargePoly::from(r)))
            .collect();

        #[cfg(feature = "parallel")]
        pk_and_randomizer.par_iter_mut().for_each(|(pk, r)| {
            pk.v0 = pk.v0 * *r;
            pk.v1 = pk.v1 * *r;
        });

        #[cfg(not(feature = "parallel"))]
        pk_and_randomizer.iter_mut().for_each(|(pk, r)| {
            pk.v0 = pk.v0 * *r;
            pk.v1 = pk.v1 * *r;
        });

        let mut agg_pk = pk_and_randomizer[0].0;

        for (pk, _r) in pk_and_randomizer.iter().skip(1) {
            agg_pk.v0 = agg_pk.v0 + pk.v0;
            agg_pk.v1 = agg_pk.v1 + pk.v1;
        }
        agg_pk
    }
}

// HOTS public key
#[derive(Debug, Default, Clone, Copy)]
pub struct RandomizedHOTSPK {
    pub(crate) v0: [SignedPoly; LARGE_MODULUS_BITS],
    pub(crate) v1: [SignedPoly; LARGE_MODULUS_BITS],
    pub(crate) is_randomized: bool,
}

impl From<&HotsPK> for RandomizedHOTSPK {
    fn from(pk: &HotsPK) -> Self {
        RandomizedHOTSPK {
            v0: pk.v0.decompose(),
            v1: pk.v1.decompose(),
            is_randomized: false,
        }
    }
}

impl From<&RandomizedHOTSPK> for HotsPK {
    fn from(pk: &RandomizedHOTSPK) -> Self {
        HotsPK {
            v0: LargePoly::projection(&pk.v0),
            v1: LargePoly::projection(&pk.v1),
        }
    }
}

impl RandomizedHOTSPK {
    pub(crate) fn randomize_with(&mut self, ternary: &SignedPoly) {
        if self.is_randomized {
            panic!("already randomized")
        }
        let ternary_coeffs: TerPolyCoeffEncoding = ternary.into();

        #[cfg(not(feature = "parallel"))]
        self.v0.iter_mut().for_each(|x| {
            *x = SignedPoly::ter_mul_bin(&ternary_coeffs, x);
        });
        #[cfg(not(feature = "parallel"))]
        self.v1.iter_mut().for_each(|x| {
            *x = SignedPoly::ter_mul_bin(&ternary_coeffs, x);
        });

        #[cfg(feature = "parallel")]
        self.v0.par_iter_mut().for_each(|x| {
            *x = SignedPoly::ter_mul_bin(&ternary_coeffs, x);
        });
        #[cfg(feature = "parallel")]
        self.v1.par_iter_mut().for_each(|x| {
            *x = SignedPoly::ter_mul_bin(&ternary_coeffs, x);
        });

        self.is_randomized = true;
    }

    pub(crate) fn digest(&self, hasher: &HOTSHash) -> SmallPoly {
        hasher.hash_separate_inputs(&self.v0, &self.v1)
    }

    /// Aggregate multiple PKs into a single PK
    pub(crate) fn aggregate(pks: &[Self], roots: &[SmallPoly]) -> Self {
        // get and apply the randomizers
        let randomizers = Randomizers::from_pks(roots);
        Self::aggregate_with_randomizers(pks, &randomizers)
    }

    /// Aggregate a set of pks with randomizes
    pub(crate) fn aggregate_with_randomizers(pks: &[Self], randomizers: &Randomizers) -> Self {
        let mut randomized_pks: Vec<Self> = pks.to_vec();
        randomized_pks
            .iter_mut()
            .zip(randomizers.poly.iter())
            .for_each(|(x, r)| x.randomize_with(r));

        let mut res = randomized_pks[0];
        randomized_pks.iter().skip(1).for_each(|x| res += *x);
        res
    }
}

impl AddAssign for RandomizedHOTSPK {
    // Coefficient wise additions without mod reduction.
    fn add_assign(&mut self, other: Self) {
        self.v0
            .iter_mut()
            .zip(other.v0.iter())
            .for_each(|(x, y)| *x += *y);
        self.v1
            .iter_mut()
            .zip(other.v1.iter())
            .for_each(|(x, y)| *x += *y);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{hots::HomomorphicOneTimeSignature, HOTS};
    use rand::{RngCore, SeedableRng};
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn test_homomorphic_hash() {
        let mut seed = [0u8; 32];
        let mut rng = ChaCha20Rng::from_seed(seed);
        let pp = HOTS::setup(&mut rng);
        let hasher = HOTSHash::init(&mut rng);

        for _ in 0..10 {
            rng.fill_bytes(&mut seed);
            let mut pks = Vec::new();
            let mut pks_randomized = Vec::new();
            let mut roots = Vec::new();
            let mut digests = Vec::new();

            for counter in 0..1 {
                let (pk, _sk) = HOTS::key_gen(&seed, counter, &pp);
                let rand_pk = RandomizedHOTSPK::from(&pk);
                let digest = rand_pk.digest(&hasher);

                pks_randomized.push(rand_pk);
                pks.push(pk);
                digests.push(digest);
                roots.push(SmallPoly::rand_poly(&mut rng));
            }
            let randomizers = Randomizers::from_pks(&roots);
            let agg_pk_randomized = RandomizedHOTSPK::aggregate(&pks_randomized, &roots);
            let agg_digest = agg_pk_randomized.digest(&hasher);
            let mut agg_digest_rec = SmallPoly::default();

            for (&d, r) in digests.iter().zip(randomizers.poly.iter()) {
                agg_digest_rec += d * SmallPoly::from(r);
            }
            assert_eq!(agg_digest, agg_digest_rec);
        }
    }
}
