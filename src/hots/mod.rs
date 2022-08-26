mod hash;
mod pk;
mod sig;

use crate::poly::LargeNTTPoly;
use crate::poly::LargePoly;
use crate::poly::SmallPoly;
use crate::randomizer::Randomizers;
use crate::SignedPoly;
use crate::BETA_S;
use crate::GAMMA;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use sha2::Digest;

pub use hash::HOTSHash;
pub use pk::HotsPK;
pub use pk::RandomizedHOTSPK;
pub use sig::HotsSig;

pub struct HOTS;

// HOTS public parameters
#[derive(Debug, Clone, Copy)]
pub struct HotsParam {
    pub(crate) a: [LargeNTTPoly; GAMMA],
}

// HOTS secret key
#[derive(Debug, Clone, Copy)]
pub struct HotsSK {
    pub(crate) s0: [LargeNTTPoly; GAMMA],
    pub(crate) s1: [LargeNTTPoly; GAMMA],
}

pub trait HomomorphicOneTimeSignature {
    type Param;
    type PK;
    type SK;
    type Signature;

    fn setup<R: Rng>(rng: &mut R) -> Self::Param;

    fn derive_sk(seed: &[u8; 32], counter: usize) -> Self::SK;

    fn key_gen(seed: &[u8; 32], counter: usize, pp: &Self::Param) -> (Self::PK, Self::SK);

    fn sign(sk: &Self::SK, message: &[u8]) -> Self::Signature;

    fn verify(pk: &Self::PK, message: &[u8], sig: &Self::Signature, pp: &Self::Param) -> bool;

    fn aggregate(sigs: &[Self::Signature], roots: &[SmallPoly]) -> Self::Signature;

    fn batch_verify(
        pks: &[Self::PK],
        message: &[u8],
        sig: &Self::Signature,
        roots: &[SmallPoly],
        pp: &Self::Param,
    ) -> bool;
}

impl HomomorphicOneTimeSignature for HOTS {
    type Param = HotsParam;
    type PK = HotsPK;
    type SK = HotsSK;
    type Signature = HotsSig;

    fn setup<R: Rng>(rng: &mut R) -> Self::Param {
        let mut a = [LargeNTTPoly::default(); GAMMA];
        a.iter_mut()
            .for_each(|x| *x = LargeNTTPoly::from(&LargePoly::rand_poly(rng)));

        Self::Param { a }
    }

    fn derive_sk(seed: &[u8; 32], counter: usize) -> Self::SK {
        // initialize the rng with seed and counter
        let seed = [seed.as_ref(), counter.to_be_bytes().as_ref()].concat();
        let mut hasher = sha2::Sha256::new();
        hasher.update(seed);
        let seed = hasher.finalize().into();
        let mut rng = ChaCha20Rng::from_seed(seed);

        // sample the secret key
        let mut s0 = [LargeNTTPoly::default(); GAMMA];
        let mut s1 = [LargeNTTPoly::default(); GAMMA];

        s0.iter_mut()
            .for_each(|x| *x = LargeNTTPoly::from(&SignedPoly::rand_mod_beta_s(&mut rng)));
        s1.iter_mut().for_each(|x| {
            *x = LargeNTTPoly::from(&SignedPoly::rand_fixed_weight_ternary(&mut rng, BETA_S))
        });
        Self::SK { s0, s1 }
    }

    fn key_gen(seed: &[u8; 32], counter: usize, pp: &Self::Param) -> (Self::PK, Self::SK) {
        let sk = Self::derive_sk(seed, counter);

        // build the pk
        let mut pk = HotsPK::default();

        pp.a.iter()
            .zip(sk.s0.iter().zip(sk.s1.iter()))
            .for_each(|(&a, (&s0, &s1))| {
                pk.v0 += (&(a * s0)).into();
                pk.v1 += (&(a * s1)).into();
            });

        (pk, sk)
    }

    fn sign(sk: &Self::SK, message: &[u8]) -> Self::Signature {
        let mut sigma = [LargePoly::default(); GAMMA];
        let hm: LargeNTTPoly = (&SignedPoly::from_hash_message(message)).into();
        for (s, (&s0, &s1)) in sigma.iter_mut().zip(sk.s0.iter().zip(sk.s1.iter())) {
            *s = (&(s0 * hm + s1)).into();
        }
        HotsSig {
            sigma,
            is_randomized: false,
        }
    }

    fn verify(pk: &Self::PK, message: &[u8], sig: &Self::Signature, pp: &Self::Param) -> bool {
        //todo: check norm of signature
        let hm: LargeNTTPoly = (&SignedPoly::from_hash_message(message)).into();
        let mut left = LargeNTTPoly::default();
        for (&a, s) in pp.a.iter().zip(sig.sigma.iter()) {
            left += a * LargeNTTPoly::from(s)
        }
        let right = hm * LargeNTTPoly::from(&pk.v0) + LargeNTTPoly::from(&pk.v1);
        // println!("left {:?}", left);
        // println!("right {:?}", right);
        left == right
    }

    fn aggregate(sigs: &[Self::Signature], roots: &[SmallPoly]) -> Self::Signature {
        // check that length are correct
        assert_eq!(sigs.len(), roots.len());

        let randomizers = Randomizers::from_pks(roots);
        Self::Signature::aggregate_with_randomizers(sigs, &randomizers)
    }

    fn batch_verify(
        pks: &[Self::PK],
        message: &[u8],
        sig: &Self::Signature,
        roots: &[SmallPoly],
        pp: &Self::Param,
    ) -> bool {
        // check that length are correct
        assert_eq!(pks.len(), roots.len());

        let agg_pk = Self::PK::aggregate(pks, roots);
        Self::verify(&agg_pk, message, sig, pp)
    }
}

pub(crate) fn batch_verify_with_aggregated_pk(
    agg_pk: &RandomizedHOTSPK,
    message: &[u8],
    agg_sig: &HotsSig,
    pp: &HotsParam,
) -> bool {
    let agg_pk: HotsPK = agg_pk.into();
    HOTS::verify(&agg_pk, message, agg_sig, pp)
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::RngCore;

    #[test]
    fn test_hots() {
        let message = "this is the message to sign";
        let mut seed = [0u8; 32];
        let mut rng = ChaCha20Rng::from_seed(seed);
        let pp = HOTS::setup(&mut rng);

        for _ in 0..10 {
            rng.fill_bytes(&mut seed);
            let mut pks = Vec::new();
            let mut sigs = Vec::new();
            let mut roots = Vec::new();

            for counter in 0..100 {
                let (pk, sk) = HOTS::key_gen(&seed, counter, &pp);
                let sig = HOTS::sign(&sk, message.as_ref());
                assert!(HOTS::verify(&pk, message.as_ref(), &sig, &pp));
                pks.push(pk);
                sigs.push(sig);
                roots.push(SmallPoly::rand_poly(&mut rng));
            }
            let agg_sig = HOTS::aggregate(&sigs, &roots);
            assert!(HOTS::batch_verify(
                &pks,
                message.as_ref(),
                &agg_sig,
                &roots,
                &pp
            ));
            let pk_randomized: Vec<RandomizedHOTSPK> = pks.iter().map(|x| x.into()).collect();
            let agg_pk_randomized = RandomizedHOTSPK::aggregate(&pk_randomized, &roots);
            batch_verify_with_aggregated_pk(&agg_pk_randomized, message.as_ref(), &agg_sig, &pp);
        }
    }
}
