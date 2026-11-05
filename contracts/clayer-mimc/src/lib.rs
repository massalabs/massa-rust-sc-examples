//! C-Layer specific ABI: the MiMC hash over BN254 (`assembly_script_hash_mimc`),
//! used for zero-knowledge friendly commitments.
//!
//! Importing `clayer::mimc` makes the module instantiable on C-Layer nodes
//! only (Massa nodes lack the import): deploy this contract on a C-Layer
//! network or a C-Layer sandbox node.
#![cfg_attr(target_arch = "wasm32", no_std)]

use massa_sc_sdk::crypto::to_hex;
use massa_sc_sdk::prelude::*;

const COMMITMENTS_PREFIX: &[u8] = b"c:";

#[export]
pub fn constructor(_args: Args) {
    assert!(context::is_deploying_contract(), "constructor can only be called at deployment");
    generate_event(&format!("deployed on chain {} (C-Layer: {})", context::chain_id(), context::is_clayer()));
}

/// `mimc(data: bytes) -> bytes(32)`: raw hash of `data` (< 32 bytes, or a
/// multiple of 32 bytes of field elements).
#[export]
pub fn mimc(mut args: Args) -> [u8; 32] {
    let data: Vec<u8> = args.must_next("data");
    let h = clayer::mimc(&data);
    generate_event(&format!("mimc({})=0x{}", to_hex(&data), to_hex(&h)));
    h
}

/// `mimc_chain(left: bytes(32), right: bytes(32)) -> bytes(32)`: hash of the
/// concatenation of two field elements, the Merkle-tree building block.
#[export]
pub fn mimc_chain(mut args: Args) -> [u8; 32] {
    let left: Vec<u8> = args.must_next("left");
    let right: Vec<u8> = args.must_next("right");
    assert!(left.len() == 32 && right.len() == 32, "inputs must be 32-byte field elements");
    let mut input = left;
    input.extend_from_slice(&right);
    clayer::mimc(&input)
}

/// `commit(secret: bytes) -> bytes(32)`: stores `mimc(secret)` under the
/// caller and returns it.
#[export]
pub fn commit(mut args: Args) -> [u8; 32] {
    let secret: Vec<u8> = args.must_next("secret");
    let h = clayer::mimc(&secret);
    let mut key = COMMITMENTS_PREFIX.to_vec();
    key.extend_from_slice(context::caller().as_bytes());
    storage::set(key, h);
    generate_event(&format!("commitment 0x{} stored for {}", to_hex(&h), context::caller()));
    h
}

/// `reveal(secret: bytes) -> bool`: checks `secret` against the caller's commitment.
#[export]
pub fn reveal(mut args: Args) -> bool {
    let secret: Vec<u8> = args.must_next("secret");
    let mut key = COMMITMENTS_PREFIX.to_vec();
    key.extend_from_slice(context::caller().as_bytes());
    let stored = storage::try_get(key).expect("no commitment for the caller");
    stored == clayer::mimc(&secret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use massa_sc_sdk::mock;

    // Vectors from the C-Layer node test-suite (gnark-crypto compatible).
    #[test]
    fn matches_node_vectors() {
        mock::reset();
        let h1 = mimc(Args::new().add(b"Hello World".to_vec()));
        assert_eq!(to_hex(&h1), "25097868b27b93cd01359a9b5f7e2a733e88b60ce32d1699b4e97b65ce87a2b8");
        assert_eq!(
            to_hex(&mimc_chain(Args::new().add(h1.to_vec()).add(h1.to_vec()))),
            "1f0dd40c969d90a1f9155e51357bacea9740f98695b3f1f13125801680e9d033"
        );
    }

    #[test]
    fn commit_reveal() {
        mock::reset();
        commit(Args::new().add(b"my secret".to_vec()));
        assert!(reveal(Args::new().add(b"my secret".to_vec())));
        assert!(!reveal(Args::new().add(b"other".to_vec())));
    }
}
