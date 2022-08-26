use crate::hots::{batch_verify_with_aggregated_pk, HomomorphicOneTimeSignature, RandomizedHOTSPK};
use crate::poly::SmallPoly;
use crate::randomizer::Randomizers;
use crate::{
    hots::{HotsParam, HotsSig},
    HOTSHash, HVCHash, MultiSig, Path, HEIGHT, HOTS,
};
use crate::{RandomizedPath, Tree};
use rand::Rng;
#[cfg(feature = "parallel")]
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

pub struct SMSigScheme;

#[derive(Debug, Clone)]
pub struct SMSigParam {
    hvc_hasher: HVCHash,
    hots_hasher: HOTSHash,
    hots_param: HotsParam,
}
#[derive(Debug, Clone)]
pub struct SMSigSK {
    sk_seed: [u8; 32],
    tree: Tree,
}

pub type SMSigPK = SmallPoly;
#[derive(Debug, Clone)]
pub struct SMSignature {
    path: RandomizedPath,
    hots_pk: RandomizedHOTSPK,
    hots_sig: HotsSig,
}

impl MultiSig for SMSigScheme {
    type Param = SMSigParam;
    type PK = SMSigPK;
    type SK = SMSigSK;
    type Signature = SMSignature;

    fn setup<R: Rng>(rng: &mut R) -> Self::Param {
        Self::Param {
            hvc_hasher: HVCHash::init(rng),
            hots_hasher: HOTSHash::init(rng),
            hots_param: HOTS::setup(rng),
        }
    }

    fn key_gen(seed: &[u8; 32], pp: &Self::Param) -> (Self::PK, Self::SK) {
        // let mut pk_digests = Vec::new();
        // for index in 0..1 << HEIGHT - 1 {
        //     let (pk, _sk) = HOTS::key_gen(seed, index, &pp.hots_param);
        //     pk_digests.push(pk.digest(&pp.hots_hasher));
        // }
        // let tree = Tree::new_with_leaf_nodes(&pk_digests, &pp.hvc_hasher);

        // (
        //     tree.root(),
        //     SMSigSK {
        //         sk_seed: *seed,
        //         tree,
        //     },
        // )

        let mut pk_digests = vec![SmallPoly::default(); 1 << (HEIGHT - 1)];

        #[cfg(not(feature = "parallel"))]
        pk_digests.iter_mut().enumerate().for_each(|(index, pkd)| {
            let (pk, _sk) = HOTS::key_gen(seed, index, &pp.hots_param);
            *pkd = pk.digest(&pp.hots_hasher)
        });

        #[cfg(feature = "parallel")]
        pk_digests
            .par_iter_mut()
            .enumerate()
            .for_each(|(index, pkd)| {
                let (pk, _sk) = HOTS::key_gen(seed, index, &pp.hots_param);
                *pkd = pk.digest(&pp.hots_hasher)
            });

        let tree = Tree::new_with_leaf_nodes(&pk_digests, &pp.hvc_hasher);

        (
            tree.root(),
            SMSigSK {
                sk_seed: *seed,
                tree,
            },
        )
    }

    fn sign(sk: &Self::SK, index: usize, message: &[u8], pp: &Self::Param) -> Self::Signature {
        let path = sk.tree.gen_proof(index);
        let (hots_pk, hots_sk) = HOTS::key_gen(&sk.sk_seed, index, &pp.hots_param);
        let hots_sig = HOTS::sign(&hots_sk, message);
        SMSignature {
            path: (&path).into(),
            hots_pk: (&hots_pk).into(),
            hots_sig,
        }
        // // SMSignature::default()
        // todo!()
    }

    fn verify(pk: &Self::PK, message: &[u8], sig: &Self::Signature, pp: &Self::Param) -> bool {
        // check signature against hots pk
        let hots_pk = (&sig.hots_pk).into();

        if !HOTS::verify(&hots_pk, message, &sig.hots_sig, &pp.hots_param) {
            return false;
        }

        // check hots public key membership
        let path = Path::from(&sig.path);
        if !path.verify(pk, &pp.hvc_hasher) {
            return false;
        }
        let pk_digest = hots_pk.digest(&pp.hots_hasher);
        if sig.path.index & 1 == 0 {
            pk_digest == path.nodes[HEIGHT - 2].0
        } else {
            pk_digest == path.nodes[HEIGHT - 2].1
        }
    }

    fn aggregate(sigs: &[Self::Signature], roots: &[SmallPoly]) -> Self::Signature {
        let randomizers = Randomizers::from_pks(roots);

        // aggregate HOTS pk
        let pks: Vec<RandomizedHOTSPK> = sigs.iter().map(|x| x.hots_pk).collect();
        let agg_pk = RandomizedHOTSPK::aggregate_with_randomizers(&pks, &randomizers);

        // aggregate HOTS sig
        let hots_sigs: Vec<HotsSig> = sigs.iter().map(|x| x.hots_sig).collect();
        let agg_sig = HotsSig::aggregate_with_randomizers(&hots_sigs, &randomizers);

        // aggregate the membership proof
        let membership_proofs: Vec<RandomizedPath> = sigs.iter().map(|x| x.path.clone()).collect();
        let agg_proof =
            RandomizedPath::aggregate_with_randomizers(&membership_proofs, &randomizers);
        Self::Signature {
            path: agg_proof,
            hots_pk: agg_pk,
            hots_sig: agg_sig,
        }
    }

    fn batch_verify(
        pks: &[Self::PK],
        message: &[u8],
        sig: &Self::Signature,
        pp: &Self::Param,
    ) -> bool {
        if !batch_verify_with_aggregated_pk(&sig.hots_pk, message, &sig.hots_sig, &pp.hots_param) {
            println!("HOTS failed");
            // return false;
        }
        if !sig.path.verify(pks, &pp.hvc_hasher) {
            println!("HVC failed");
            // return false;
        }
        if sig.path.index & 1 == 0 {
            sig.hots_pk.digest(&pp.hots_hasher)
                == SmallPoly::projection(&sig.path.nodes[HEIGHT - 2].0)
        } else {
            sig.hots_pk.digest(&pp.hots_hasher)
                == SmallPoly::projection(&sig.path.nodes[HEIGHT - 2].1)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::{RngCore, SeedableRng};
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn test_smsig() {
        let message = "this is the message to sign";
        let mut seed = [0u8; 32];
        let mut rng = ChaCha20Rng::from_seed(seed);

        let pp = SMSigScheme::setup(&mut rng);

        for _ in 0..10 {
            rng.fill_bytes(&mut seed);
            let (pk, sk) = SMSigScheme::key_gen(&seed, &pp);
            for _ in 0..10 {
                let index = rng.next_u32() % (1 << HEIGHT - 1);
                let sig = SMSigScheme::sign(&sk, index as usize, message.as_ref(), &pp);
                assert!(SMSigScheme::verify(&pk, message.as_ref(), &sig, &pp))
            }
        }

        for index in 0..10 {
            let mut sigs = Vec::new();
            let mut pks = Vec::new();
            for _ in 0..10 {
                rng.fill_bytes(&mut seed);
                let (pk, sk) = SMSigScheme::key_gen(&seed, &pp);

                let sig = SMSigScheme::sign(&sk, index as usize, message.as_ref(), &pp);
                assert!(SMSigScheme::verify(&pk, message.as_ref(), &sig, &pp));
                pks.push(pk);
                sigs.push(sig);
            }

            let agg_sig = SMSigScheme::aggregate(&sigs, &pks);
            assert!(SMSigScheme::batch_verify(
                &pks,
                message.as_ref(),
                &agg_sig,
                &pp
            ))
        }
    }
}
