use crate::{hvc::hash::HVCHash, poly::SmallPoly, randomizer::Randomizers, RandomizedPath, HEIGHT};
use core::fmt;
use std::fmt::Display;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Path {
    pub(crate) nodes: [(SmallPoly, SmallPoly); HEIGHT - 1], // left and right nodes
    pub(crate) index: usize,
}

impl Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let position_list = self.position_list();
        for ((i, (left, right)), is_right_node) in self.nodes.iter().enumerate().zip(position_list)
        {
            writeln!(
                f,
                "{}-th: left {} right {}, is right {}",
                i, left, right, is_right_node
            )?;
        }

        Ok(())
    }
}

impl Path {
    /// The position of on_path node in `leaf_and_sibling_hash` and `non_leaf_and_sibling_hash_path`.
    /// `position[i]` is 0 (false) iff `i`th on-path node from top to bottom is on the left.
    ///
    /// This function simply converts `self.leaf_index` to boolean array in big endian form.
    fn position_list(&'_ self) -> impl '_ + Iterator<Item = bool> {
        (0..self.nodes.len() + 1)
            .map(move |i| ((self.index >> i) & 1) != 0)
            .rev()
    }

    /// verifies the path against a root
    pub fn verify(&self, root: &SmallPoly, hasher: &HVCHash) -> bool {
        // check that the first two elements hashes to root
        if hasher.decom_then_hash(&self.nodes[0].0, &self.nodes[0].1) != *root {
            return false;
        }
        let position_list = self.position_list();

        for ((i, (left, right)), is_right_node) in
            self.nodes.iter().enumerate().zip(position_list).skip(1)
        {
            let digest = hasher.decom_then_hash(left, right);
            if is_right_node {
                if digest != self.nodes[i - 1].1 {
                    return false;
                }
            } else if digest != self.nodes[i - 1].0 {
                return false;
            }
        }

        true
    }

    pub(crate) fn aggregate_with_randomizers(
        paths: &[Self],
        randomizers: &Randomizers,
    ) -> RandomizedPath {
        // check that length are correct
        let len = paths.len();
        assert_eq!(len, randomizers.poly.len());
        // check that we aggregate for a same index
        for e in paths.iter().skip(1) {
            assert_eq!(e.index, paths[0].index)
        }

        let randomized_paths: Vec<_> = paths.iter().map(|x| x.into()).collect();
        RandomizedPath::aggregate_with_randomizers(&randomized_paths, randomizers)
    }

    /// Aggregate a set of paths
    pub fn aggregation(paths: &[Self], roots: &[SmallPoly]) -> RandomizedPath {
        // get and apply the randomizers
        let randomizers = Randomizers::from_pks(roots);
        Self::aggregate_with_randomizers(paths, &randomizers)
    }

    pub fn random_for_testing<R: rand::Rng>(rng: &mut R, hasher: &HVCHash) -> (Self, SmallPoly) {
        let mut nodes = vec![(SmallPoly::rand_poly(rng), SmallPoly::rand_poly(rng))];

        for i in 1..HEIGHT - 1 {
            let left = hasher.decom_then_hash(&nodes[i - 1].0, &nodes[i - 1].1);
            nodes.push((left, SmallPoly::rand_poly(rng)))
        }
        let root = hasher.decom_then_hash(&nodes[HEIGHT - 2].0, &nodes[HEIGHT - 2].1);
        nodes.reverse();
        let mut path = Self::default();
        path.nodes.clone_from_slice(&nodes);
        path.index = 0;

        (path, root)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn test_path() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
        let hasher = HVCHash::init(&mut rng);
        for _ in 0..100 {
            let (path, root) = Path::random_for_testing(&mut rng, &hasher);
            assert!(path.verify(&root, &hasher))
        }
    }

    #[test]
    fn test_path_conversion() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
        let hasher = HVCHash::init(&mut rng);
        for _ in 0..100 {
            let (path, _root) = Path::random_for_testing(&mut rng, &hasher);
            let randomize_path: RandomizedPath = (&path).into();
            let path_rec = (&randomize_path).into();
            assert_eq!(path, path_rec)
        }
    }

    #[test]
    fn test_randomized_path() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
        let hasher = HVCHash::init(&mut rng);
        let mut paths = vec![];
        let mut roots = vec![];
        for _ in 0..100 {
            let (path, root) = Path::random_for_testing(&mut rng, &hasher);
            assert!(path.verify(&root, &hasher));
            paths.push(path);
            roots.push(root);
        }

        let path = Path::aggregation(&paths, &roots);

        assert!(path.verify(&roots, &hasher))
    }
}
