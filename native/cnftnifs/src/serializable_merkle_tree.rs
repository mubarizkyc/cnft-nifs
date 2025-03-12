use serde::{Deserialize, Serialize};
use spl_merkle_tree_reference::MerkleTree;
// used to encode & decode spl_merkle_tree_reference::MerkleTree
#[derive(Serialize, Deserialize)]
pub struct SerializableMerkleTree {
    pub leaf_nodes: Vec<[u8; 32]>, // Matches Node type
    pub root: [u8; 32],            // Matches MerkleTree root
}

impl<'a> From<&'a MerkleTree> for SerializableMerkleTree {
    fn from(tree: &'a MerkleTree) -> Self {
        SerializableMerkleTree {
            leaf_nodes: tree.leaf_nodes.iter().map(|n| n.borrow().node).collect(), // ✅ Extract [u8; 32]
            root: tree.root,                                                       // ✅ Correct
        }
    }
}
