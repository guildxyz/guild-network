use crate::hash::{Hash, Keccak256};
use crate::identity::Identity;
use crate::{Decode, Encode, SpVec, TypeInfo};
use serde::{Deserialize, Serialize};

pub use binary_merkle_tree::merkle_root as root;
pub type Leaf<'a> = binary_merkle_tree::Leaf<'a, Hash>;

#[derive(Serialize, Deserialize, Encode, Decode, TypeInfo, Clone, Debug, Eq, PartialEq)]
pub struct Proof {
    pub path: SpVec<Hash>,
    pub id_index: u8,
}

impl Proof {
    pub fn new(allowlist: &[Identity], leaf_index: usize, id_index: u8) -> Self {
        let merkle_proof =
            binary_merkle_tree::merkle_proof::<Keccak256, _, _>(allowlist, leaf_index);
        Self {
            path: merkle_proof.proof,
            id_index,
        }
    }

    // NOTE leaf index does not play a role in verification
    // only a bound check is performed to ensure the index is
    // less than the number of leaves in the tree. Setting the
    // index to 0 ensures that it will never be greater. If the
    // number of leaves is 0, though, access will still be
    // denied as the proof evaluates to false. However, empty
    // allowlists are not allowed in the respective
    // 'create_role' call
    pub fn verify(&self, root: &Hash, n_leaves: usize, leaf: Leaf) -> bool {
        binary_merkle_tree::verify_proof::<'_, Keccak256, _, _>(
            root,
            self.path.iter().copied(),
            n_leaves,
            0,
            leaf,
        )
    }
}
