//! Cryptographic helpers offered by the node: hashes, Massa and EVM signature
//! verification, public-key-to-address conversion, address validation.
//! Entry points return hex strings so results are readable in events and
//! read-only calls.
#![cfg_attr(target_arch = "wasm32", no_std)]

use massa_sc_sdk::crypto::{from_hex, to_hex};
use massa_sc_sdk::prelude::*;

#[export]
pub fn constructor(_args: Args) {
    assert!(context::is_deploying_contract(), "constructor can only be called at deployment");
}

/// `hashes(data: bytes) -> string`: `"blake3=<hex>;sha256=<hex>;keccak256=<hex>"`
#[export]
pub fn hashes(mut args: Args) -> String {
    let data: Vec<u8> = args.must_next("data");
    let out = format!(
        "blake3={};sha256={};keccak256={}",
        to_hex(&crypto::blake3(&data)),
        to_hex(&crypto::sha256(&data)),
        to_hex(&crypto::keccak256(&data))
    );
    generate_event(&out);
    out
}

/// `verify_massa(public_key: string, data: string, signature: string) -> bool`
/// with base58check public key and signature, as produced by `massa-web3`
/// `account.sign(bytes)`.
#[export]
pub fn verify_massa(mut args: Args) -> bool {
    let public_key: String = args.must_next("public_key");
    let data: String = args.must_next("data");
    let signature: String = args.must_next("signature");
    let ok = crypto::is_signature_valid(&public_key, &data, &signature);
    generate_event(&format!("massa signature valid: {ok}"));
    ok
}

/// `verify_evm(hash: bytes, signature: bytes) -> string`: recovers the signer
/// of an Ethereum signature (65 bytes, `r || s || v`) over a 32-byte digest
/// and returns its 20-byte address as hex. Also checks the signature against
/// the recovered key.
#[export]
pub fn verify_evm(mut args: Args) -> String {
    let hash: Vec<u8> = args.must_next("hash");
    let signature: Vec<u8> = args.must_next("signature");
    let public_key = crypto::evm_get_pubkey_from_signature(&hash, &signature);
    let address = crypto::evm_get_address_from_pubkey(&public_key);
    let valid = crypto::is_evm_signature_valid(&hash, &signature, &public_key);
    let out = format!("0x{} valid={valid}", to_hex(&address));
    generate_event(&out);
    out
}

/// `address_info(address: string) -> string`
#[export]
pub fn address_info(mut args: Args) -> String {
    let address: Address = args.must_next("address");
    let kind = if !address.is_valid() {
        "invalid"
    } else if address.is_eoa() {
        "user account"
    } else {
        "smart contract"
    };
    format!("{address}: {kind}, balance {} nanoMAS", address.balance())
}

/// `public_key_to_address(public_key: string) -> string`
#[export]
pub fn public_key_to_address(mut args: Args) -> Address {
    let public_key: String = args.must_next("public_key");
    crypto::public_key_to_address(&public_key)
}

/// `hex_roundtrip(hex: string) -> bytes`
#[export]
pub fn hex_roundtrip(mut args: Args) -> Vec<u8> {
    let hex: String = args.must_next("hex");
    from_hex(&hex).expect("invalid hex")
}

#[cfg(test)]
mod tests {
    use super::*;
    use massa_sc_sdk::mock;

    #[test]
    fn hash_vectors() {
        mock::reset();
        let out = hashes(Args::new().add(b"abc".to_vec()));
        assert!(out.contains("sha256=ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"));
        assert!(out.contains("keccak256=4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45"));
    }

    #[test]
    fn evm_recovery_shape() {
        mock::reset();
        mock::set_evm_recovered_pubkey(vec![1u8; 64]);
        let out = verify_evm(Args::new().add(vec![0u8; 32]).add(vec![0u8; 65]));
        assert!(out.starts_with("0x") && out.contains("valid=true"));
        assert_eq!(hex_roundtrip(Args::new().add("0a0b")), vec![10, 11]);
    }

    #[test]
    fn addresses() {
        mock::reset();
        let info = address_info(Args::new().add(context::caller()));
        assert!(info.contains("user account"));
        assert_eq!(address_info(Args::new().add("nope")), "nope: invalid, balance 0 nanoMAS");
    }
}
