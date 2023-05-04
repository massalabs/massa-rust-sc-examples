// ****************************************************************************
// needed to override the panic handler
#![no_std]
// ****************************************************************************

// ****************************************************************************
// Imports from massa-rust-sc-sdk
// ****************************************************************************
use massa_rust_sc_sdk as sdk;
// as we are in a no_std environment, the sdk redifined the allocators for
// default containers, use 'use sdk::*' to get them
use sdk::*;
// Imports what is needed from the SDK
use sdk::abis::{call, create_sc, generate_event, log, transfer_coins};
// ****************************************************************************

// ****************************************************************************
// Simple Smart Contract that generate an event
// ****************************************************************************

fn create_contract() -> Result<String, String> {
    // const bytes: StaticArray<u8> =
    // fileToByteArray("./build/release.wasm_add");
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../target/wasm32-unknown-unknown/debug/massa_rust_sc_generate_event.wasm_add"
    )).to_vec();

    log("calling create_sc".to_owned());
    let sc_address = create_sc(module)?;

    log("SC created @:".to_string() + &sc_address);

    log("Will transfer coins: 100_000_000_000".to_owned());
    transfer_coins(sc_address.clone(), 100);
    log("Coins transfered".to_owned());

    Ok(sc_address)
}

#[no_mangle]
fn main(_args: u32) -> u32 {
    log("SC main".to_string());

    match create_contract() {
        Ok(sc_address) => {
            let _ret = call(
                sc_address.clone(),
                "call_generate_event".to_owned(),
                Vec::new(),
                0,
            );
            generate_event(
                "Created a Protobuffed smart-contract at:".to_string()
                    + &sc_address,
            );
        }
        Err(e) => {
            panic!("Error creating SC: {}", e)
        }
    }

    // data MUST be returned this way
    encode_length_prefixed(vec![0xC, 0xA, 0xF, 0xE])
}

#[cfg(test)]
mod tests {
    use crate::call_generate_event;

    #[test]
    fn test_main() {
        assert_eq!(0, call_generate_event(0));
    }
}
