#![allow(dead_code)]

mod hots;
mod hvc;
mod param;
mod poly;
mod randomizer;
mod smsig;

pub use hots::HOTSHash;
pub use hots::HOTS;
pub use hvc::HVCHash;
pub use hvc::RandomizedPath;
pub use hvc::{Path, Tree};
pub use param::*;
pub use poly::SignedPoly;
pub use poly::*;
pub use randomizer::Randomizers;
pub use smsig::SMSigScheme;

use rand::Rng;

pub trait MultiSig {
    type Param;
    type PK;
    type SK;
    type Signature;

    fn setup<R: Rng>(rng: &mut R) -> Self::Param;

    fn key_gen(seed: &[u8; 32], pp: &Self::Param) -> (Self::PK, Self::SK);

    fn sign(sk: &Self::SK, index: usize, message: &[u8], pp: &Self::Param) -> Self::Signature;

    fn verify(pk: &Self::PK, message: &[u8], sig: &Self::Signature, pp: &Self::Param) -> bool;

    fn aggregate(sigs: &[Self::Signature], roots: &[SmallPoly]) -> Self::Signature;

    fn batch_verify(
        pks: &[Self::PK],
        message: &[u8],
        sig: &Self::Signature,
        pp: &Self::Param,
    ) -> bool;
}
