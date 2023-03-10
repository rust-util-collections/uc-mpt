//!
//!
//!

mod nibbles;
mod node;
mod tests;

mod db;
mod errors;
mod trie;

pub use db::{MemoryDB, DB};
pub use errors::{MemDBError, TrieError};
pub use trie::{PatriciaTrie, Trie};
pub use verify::verify_proof;

const DIGEST_LEN: usize = blake3::OUT_LEN;

#[inline(always)]
fn digest(data: &[u8]) -> [u8; DIGEST_LEN] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(data);
    hasher.finalize().into()
}

mod verify {
    use crate::{
        digest, trie::TrieResult, MemoryDB, PatriciaTrie, Trie, TrieError, DB,
        DIGEST_LEN,
    };
    use std::sync::Arc;

    pub fn verify_proof(
        root_hash: &[u8],
        key: &[u8],
        proof: Vec<Vec<u8>>,
    ) -> TrieResult<Option<Vec<u8>>> {
        let memdb = Arc::new(MemoryDB::new());
        for node_encoded in proof.into_iter() {
            let hash = digest(&node_encoded);

            if root_hash.eq(&hash) || node_encoded.len() >= DIGEST_LEN {
                memdb.insert(&hash, &node_encoded).unwrap();
            }
        }

        PatriciaTrie::from(memdb, root_hash)
            .or(Err(TrieError::InvalidProof))?
            .get(key)
            .or(Err(TrieError::InvalidProof))
    }
}
