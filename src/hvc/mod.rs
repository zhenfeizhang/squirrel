mod hash;
mod tree;

pub use hash::HVCHash;
pub use tree::path::Path;
pub use tree::randomize_path::RandomizedPath;
pub use tree::Tree;

use crate::poly::SmallPoly;

pub trait HomomorphicVectorCommitment {
    type Hasher;
    type Commitment;
    type Node;
    type MembershipProof;
    type AggregatedProof;

    fn commit(hasher: &Self::Hasher, msg: &[Self::Node]) -> Self;

    fn open(&self, index: usize) -> Self::MembershipProof;

    fn verify_single(
        hasher: &Self::Hasher,
        proof: &Self::MembershipProof,
        root: &Self::Commitment,
    ) -> bool;

    fn verify_aggregated(
        hasher: &Self::Hasher,
        proof: &Self::AggregatedProof,
        root: &[Self::Commitment],
    ) -> bool;

    fn aggregate(
        proofs: &[Self::MembershipProof],
        roots: &[Self::Commitment],
    ) -> Self::AggregatedProof;
}

pub struct HVC(pub(crate) Tree);

impl HomomorphicVectorCommitment for HVC {
    type Hasher = HVCHash;
    type Commitment = SmallPoly;
    type Node = SmallPoly;
    type MembershipProof = Path;
    type AggregatedProof = RandomizedPath;

    fn commit(hasher: &Self::Hasher, msg: &[Self::Node]) -> Self {
        Self(Tree::new_with_leaf_nodes(msg, hasher))
    }

    fn open(&self, index: usize) -> Self::MembershipProof {
        self.0.gen_proof(index)
    }

    fn verify_single(
        hasher: &Self::Hasher,
        proof: &Self::MembershipProof,
        root: &Self::Commitment,
    ) -> bool {
        proof.verify(root, hasher)
    }

    fn verify_aggregated(
        hasher: &Self::Hasher,
        proof: &Self::AggregatedProof,
        roots: &[Self::Commitment],
    ) -> bool {
        proof.verify(roots, hasher)
    }

    fn aggregate(
        proofs: &[Self::MembershipProof],
        roots: &[Self::Commitment],
    ) -> Self::AggregatedProof {
        Path::aggregation(proofs, roots)
    }
}
