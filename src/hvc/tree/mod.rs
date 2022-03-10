//! This file implements the tree construction.
//! This file is adopted from <https://github.com/arkworks-rs/crypto-primitives/blob/main/src/merkle_tree/mod.rs>
//! with substantial changes.
//!
//!
//!

pub mod path;
pub mod randomize_path;

use super::hash::HVCHash;
use crate::{poly::SmallPoly, Path, HEIGHT};
use core::fmt;
use std::fmt::Display;

#[derive(Clone, Debug, Default)]
pub struct Tree {
    /// stores the non-leaf nodes in level order. The first element is the root node.
    /// The ith nodes (starting at 1st) children are at indices `2*i`, `2*i+1`
    non_leaf_nodes: Vec<SmallPoly>,

    /// store the hash of leaf nodes from left to right
    leaf_nodes: Vec<SmallPoly>,
}

impl Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "non leaf nodes:")?;
        for (i, e) in self.non_leaf_nodes.iter().enumerate() {
            writeln!(f, "{}: {}", i, e)?;
        }
        writeln!(f, "leaf nodes:")?;
        for (i, e) in self.leaf_nodes.iter().enumerate() {
            writeln!(f, "{}: {}", i, e)?;
        }
        Ok(())
    }
}

impl Tree {
    /// create an empty tree
    pub fn init(hasher: &HVCHash) -> Self {
        let leaf_nodes = vec![SmallPoly::default(); 1 << (HEIGHT - 1)];
        Self::new_with_leaf_nodes(&leaf_nodes, hasher)
    }

    /// create a new tree with leaf nodes
    pub fn new_with_leaf_nodes(leaf_nodes: &[SmallPoly], hasher: &HVCHash) -> Self {
        let len = leaf_nodes.len();
        assert_eq!(len, 1 << (HEIGHT - 1), "incorrect leaf size");

        let mut non_leaf_nodes = vec![SmallPoly::default(); (1 << (HEIGHT - 1)) - 1];

        // Compute the starting indices for each non-leaf level of the tree
        let mut index = 0;
        let mut level_indices = Vec::with_capacity(HEIGHT - 1);
        for _ in 0..(HEIGHT - 1) {
            level_indices.push(index);
            index = left_child_index(index);
        }

        // compute the hash values for the non-leaf bottom layer
        {
            let start_index = level_indices.pop().unwrap();
            let upper_bound = left_child_index(start_index);
            for current_index in start_index..upper_bound {
                // `left_child_index(current_index)` and `right_child_index(current_index) returns the position of
                // leaf in the whole tree (represented as a list in level order). We need to shift it
                // by `-upper_bound` to get the index in `leaf_nodes` list.
                let left_leaf_index = left_child_index(current_index) - upper_bound;
                let right_leaf_index = right_child_index(current_index) - upper_bound;
                // compute hash
                non_leaf_nodes[current_index] = hasher
                    .decom_then_hash(&leaf_nodes[left_leaf_index], &leaf_nodes[right_leaf_index]);
            }
        }

        // compute the hash values for nodes in every other layer in the tree
        level_indices.reverse();
        for &start_index in &level_indices {
            // The layer beginning `start_index` ends at `upper_bound` (exclusive).
            let upper_bound = left_child_index(start_index);
            for current_index in start_index..upper_bound {
                let left_index = left_child_index(current_index);
                let right_index = right_child_index(current_index);
                non_leaf_nodes[current_index] = hasher
                    .decom_then_hash(&non_leaf_nodes[left_index], &non_leaf_nodes[right_index]);
            }
        }

        Self {
            non_leaf_nodes,
            leaf_nodes: leaf_nodes.to_vec(),
        }
    }

    pub fn root(&self) -> SmallPoly {
        self.non_leaf_nodes[0]
    }

    // generate a membership proof for the given index
    pub fn gen_proof(&self, index: usize) -> Path {
        // Get Leaf hash, and leaf sibling hash,
        let leaf_index_in_tree = convert_index_to_last_level(index, HEIGHT);

        // path.len() = `tree height - 1`, the missing elements being the root
        let mut nodes = Vec::with_capacity(HEIGHT - 1);
        if index % 2 == 0 {
            nodes.push((self.leaf_nodes[index], self.leaf_nodes[index + 1]))
        } else {
            nodes.push((self.leaf_nodes[index - 1], self.leaf_nodes[index]))
        }

        // Iterate from the bottom layer after the leaves, to the top, storing all nodes and their siblings.
        let mut current_node = parent_index(leaf_index_in_tree).unwrap();
        while current_node != 0 {
            let sibling_node = sibling_index(current_node).unwrap();
            if is_left_child(current_node) {
                nodes.push((
                    self.non_leaf_nodes[current_node],
                    self.non_leaf_nodes[sibling_node],
                ));
            } else {
                nodes.push((
                    self.non_leaf_nodes[sibling_node],
                    self.non_leaf_nodes[current_node],
                ));
            }
            current_node = parent_index(current_node).unwrap();
        }

        // we want to make path from root to bottom
        nodes.reverse();
        let mut path = Path::default();
        path.index = index;
        path.nodes.clone_from_slice(&nodes);
        path
    }
}

/// Returns the index of the sibling, given an index.
#[inline]
fn sibling_index(index: usize) -> Option<usize> {
    if index == 0 {
        None
    } else if is_left_child(index) {
        Some(index + 1)
    } else {
        Some(index - 1)
    }
}

/// Returns the index of the parent, given an index.
#[inline]
fn parent_index(index: usize) -> Option<usize> {
    if index > 0 {
        Some((index - 1) >> 1)
    } else {
        None
    }
}

/// Returns the index of the left child, given an index.
#[inline]
fn left_child_index(index: usize) -> usize {
    2 * index + 1
}

/// Returns the index of the right child, given an index.
#[inline]
fn right_child_index(index: usize) -> usize {
    2 * index + 2
}

#[inline]
fn convert_index_to_last_level(index: usize, tree_height: usize) -> usize {
    index + (1 << (tree_height - 1)) - 1
}

/// Returns true iff the given index represents a left child.
#[inline]
fn is_left_child(index: usize) -> bool {
    index % 2 == 1
}

#[cfg(test)]
mod test {
    use rand::{RngCore, SeedableRng};
    use rand_chacha::ChaCha20Rng;

    use super::*;
    #[test]
    fn test_tree() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
        let hasher = HVCHash::init(&mut rng);

        let leafs: Vec<SmallPoly> = (0..(1 << (HEIGHT - 1)))
            .map(|_| SmallPoly::rand_poly(&mut rng))
            .collect();
        let tree = Tree::new_with_leaf_nodes(&leafs, &hasher);

        for _ in 0..100 {
            let index = rng.next_u32() % (1 << (HEIGHT - 1));
            let proof = tree.gen_proof(index as usize);
            assert!(proof.verify(&tree.non_leaf_nodes[0], &hasher));
        }
    }
}
