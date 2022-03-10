#[cfg(feature = "parallel")]
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::{
    hvc::hash::HVCHash,
    poly::{SmallPoly, TerPolyCoeffEncoding},
    randomizer::Randomizers,
    Path, SignedPoly, HEIGHT, SMALL_MODULUS_BITS,
};
use std::ops::{Add, AddAssign};

#[derive(Clone, Debug, Default, Copy)]
pub struct RandomizedPath {
    pub(crate) nodes: [(
        [SignedPoly; SMALL_MODULUS_BITS],
        [SignedPoly; SMALL_MODULUS_BITS],
    ); HEIGHT - 1],
    pub(crate) index: usize,
    pub(crate) is_randomized: bool,
}

// todo: improve the code to avoid `clone_from_slice`
impl From<&Path> for RandomizedPath {
    fn from(p: &Path) -> Self {
        // seems that the overhead of parallelizing the conversion is enormous
        // and we are better off without parallelization.
        //
        // #[cfg(feature = "parallel")]
        // let nodes: Vec<(Vec<SmallPoly>, Vec<SmallPoly>)> = p
        //     .nodes
        //     .clone()
        //     .into_par_iter()
        //     .map(|(left, right)| (left.decompose(), right.decompose()))
        //     .collect();
        //
        // #[cfg(not(feature = "parallel"))]
        let nodes: Vec<_> = p
            .nodes
            .iter()
            .map(|(left, right)| (left.decompose(), right.decompose()))
            .collect();
        let mut res = Self::default();
        res.nodes.clone_from_slice(&nodes);
        res.index = p.index;
        res.is_randomized = false;

        res
    }
}

impl From<&RandomizedPath> for Path {
    fn from(r: &RandomizedPath) -> Self {
        // #[cfg(feature = "parallel")]
        // let nodes = r
        //     .nodes
        //     .clone()
        //     .into_par_iter()
        //     .map(|(left, right)| (SmallPoly::projection(&left), SmallPoly::projection(&right)))
        //     .collect();
        //
        // #[cfg(not(feature = "parallel"))]
        let nodes: Vec<_> = r
            .nodes
            .iter()
            .map(|(left, right)| (SmallPoly::projection(left), SmallPoly::projection(right)))
            .collect();
        let mut res = Self::default();
        res.nodes.clone_from_slice(&nodes);
        res.index = r.index;
        res
    }
}

impl RandomizedPath {
    /// The position of on_path node in `leaf_and_sibling_hash` and `non_leaf_and_sibling_hash_path`.
    /// `position[i]` is 0 (false) iff `i`th on-path node from top to bottom is on the left.
    ///
    /// This function simply converts `self.leaf_index` to boolean array in big endian form.
    fn position_list(&'_ self) -> impl '_ + Iterator<Item = bool> {
        (0..self.nodes.len() + 1)
            .map(move |i| ((self.index >> i) & 1) != 0)
            .rev()
    }

    pub fn randomize_with(&mut self, ternary: &SignedPoly) {
        if self.is_randomized {
            panic!("already randomized")
        }
        let ternary_coeffs: TerPolyCoeffEncoding = ternary.into();

        #[cfg(not(feature = "parallel"))]
        self.nodes.iter_mut().for_each(|(left, right)| {
            left.iter_mut()
                .for_each(|x| *x = SignedPoly::ter_mul_bin(&ternary_coeffs, x));
            right
                .iter_mut()
                .for_each(|x| *x = SignedPoly::ter_mul_bin(&ternary_coeffs, x));
        });

        #[cfg(feature = "parallel")]
        self.nodes.par_iter_mut().for_each(|(left, right)| {
            left.par_iter_mut()
                .for_each(|x| *x = SignedPoly::ter_mul_bin(&ternary_coeffs, x));
            right
                .par_iter_mut()
                .for_each(|x| *x = SignedPoly::ter_mul_bin(&ternary_coeffs, x));
        });

        self.is_randomized = true;
    }

    pub(crate) fn aggregate_with_randomizers(paths: &[Self], randomizers: &Randomizers) -> Self {
        let mut randomized_paths: Vec<RandomizedPath> = paths.to_vec();
        randomized_paths
            .iter_mut()
            .zip(randomizers.poly.iter())
            .for_each(|(path, randomizer)| path.randomize_with(randomizer));

        // aggregate the result
        let mut res = randomized_paths[0].clone();

        randomized_paths
            .iter()
            .skip(1)
            .for_each(|target| res += target.clone());
        res
    }

    /// verifies the path against a list of root
    pub fn verify(&self, roots: &[SmallPoly], hasher: &HVCHash) -> bool {
        // recompute the root
        let randomziers = Randomizers::from_pks(roots);
        let mut root = SmallPoly::default();
        roots
            .iter()
            .zip(randomziers.poly.iter())
            .for_each(|(&rt, &rand)| root += rand.lifted_small() * rt);

        // check that the first two elements hashes to root
        if hasher.hash_separate_inputs(self.nodes[0].0.as_ref(), self.nodes[0].1.as_ref()) != root {
            return false;
        }
        let position_list = self.position_list();

        for ((i, (left, right)), is_right_node) in
            self.nodes.iter().enumerate().zip(position_list).skip(1)
        {
            let digest = hasher.hash_separate_inputs(left, right);
            if is_right_node {
                if digest != SmallPoly::projection(&self.nodes[i - 1].1) {
                    return false;
                }
            } else if digest != SmallPoly::projection(&self.nodes[i - 1].0) {
                return false;
            }
        }

        true
    }
}

impl Add for RandomizedPath {
    type Output = Self;

    // Coefficient wise additions without mod reduction.
    fn add(self, other: Self) -> Self {
        assert_eq!(self.index, other.index);

        let mut res = self;
        res.nodes
            .iter_mut()
            .zip(other.nodes.iter())
            .for_each(|(x, y)| {
                x.0.iter_mut()
                    .zip(y.0.iter())
                    .for_each(|(x0, y0)| *x0 += *y0);
                x.1.iter_mut()
                    .zip(y.1.iter())
                    .for_each(|(x1, y1)| *x1 += *y1);
            });

        res
    }
}

impl AddAssign for RandomizedPath {
    // Coefficient wise additions without mod reduction.
    fn add_assign(&mut self, other: Self) {
        assert_eq!(self.index, other.index);

        self.nodes
            .iter_mut()
            .zip(other.nodes.iter())
            .for_each(|(x, y)| {
                x.0.iter_mut()
                    .zip(y.0.iter())
                    .for_each(|(x0, y0)| *x0 += *y0);
                x.1.iter_mut()
                    .zip(y.1.iter())
                    .for_each(|(x1, y1)| *x1 += *y1);
            });
    }
}
