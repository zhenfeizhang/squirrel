use crate::poly::SmallPoly;
use crate::SignedPoly;
use crate::ALPHA;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use sha2::Digest;
use sha2::Sha256;

#[derive(Debug, PartialEq, Clone)]
pub struct Randomizers {
    pub(crate) poly: Vec<SignedPoly>,
}

impl Randomizers {
    pub(crate) fn rand<R: Rng>(rng: &mut R, n: usize) -> Self {
        Self {
            poly: (0..n)
                .map(|_| SignedPoly::rand_ternary(rng, ALPHA >> 1))
                .collect(),
        }
    }

    pub fn from_pks(roots: &[SmallPoly]) -> Self {
        // hash the roots into randomizers
        let mut input = Vec::new();
        for e in roots {
            input.extend_from_slice(e.digest().as_ref())
        }
        let mut hasher = Sha256::new();
        hasher.update(input);
        let seed = hasher.finalize();
        let mut rng = ChaCha20Rng::from_seed(seed.into());
        Self::rand(&mut rng, roots.len())
    }
}
