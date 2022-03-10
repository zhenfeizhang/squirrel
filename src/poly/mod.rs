mod large_poly;
mod signed_poly;
mod small_poly;

use crate::{ALPHA, N};

#[derive(Debug, Clone, PartialEq, Copy)]
// A signed polynomial of degree N
pub struct SignedPoly {
    pub(crate) coeffs: [i32; N],
}

#[derive(Debug, Clone, PartialEq, Copy)]
// ternary polynomials in canonical encoding
pub struct SmallPoly {
    pub(crate) coeffs: [u16; N],
}

#[derive(Debug, Clone, PartialEq, Copy)]
// ternary polynomials in canonical encoding
pub struct SmallNTTPoly {
    pub(crate) coeffs: [u16; N],
}

#[derive(Debug, Clone, PartialEq, Copy)]
// ternary polynomials in canonical encoding
pub struct LargePoly {
    pub(crate) coeffs: [u32; N],
}

#[derive(Debug, Clone, PartialEq, Copy)]
// ternary polynomials in canonical encoding
pub struct LargeNTTPoly {
    pub(crate) coeffs: [u32; N],
}

#[derive(Debug, Clone, PartialEq, Default)]
// ternary polynomials in coefficient encoding
pub struct TerPolyCoeffEncoding {
    pub(crate) indices: [usize; ALPHA],
}
