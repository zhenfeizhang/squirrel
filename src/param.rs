// degree of polynomial
pub const N: usize = 256;

// non-zero entries in a randomizer polynomial
pub const ALPHA: usize = 22;

// q for small ring, HVC modulus
pub const SMALL_MODULUS: u16 = 12289;

// log(q) for HVC modulus
pub const SMALL_MODULUS_BITS: usize = 14;

// the largest multiple of q that is smaller than 2^32
pub const SMALL_SAMPLE_THRESHOLD: u32 = 4294956344;

// height of the tree for the HVC scheme
pub const HEIGHT: usize = 21;

// q for the larger ring, HOTS modulus
pub const LARGE_MODULUS: u32 = 10571777;

// the largest multiple of q that is smaller than 2^32
pub const LARGE_SAMPLE_THRESHOLD: u32 = 4292141462;

// log(q) for HOTS modulus
pub(crate) const LARGE_MODULUS_BITS: usize = 24;

// dimension of secret keys in OTS
pub(crate) const GAMMA: usize = 43;

// norm bound for second component of HOTS secret key
// Also the number of non-zero entries for hash of message
pub(crate) const BETA_S: usize = 59;

pub(crate) const TWO_BETA_S_PLUS_ONE: u32 = 119;

// the largest multiple of (2*beta_s-1) that is smaller than 2^32
pub(crate) const BETA_S_SAMPLE_THRESHOLD: u32 = 4294967278;
