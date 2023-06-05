// ****************************************************************************
// needed to override the panic handler
#![no_std]
// ****************************************************************************

// ****************************************************************************
// Imports from massa-rust-sc-sdk
// ****************************************************************************
use massa_rust_sc_sdk as sdk;

// as we are in a no_std environment, the sdk redefined the allocators for
// default containers, use 'use sdk::*' to get them
use sdk::*;
// Imports what is needed from the SDK
use sdk::abis::{log, Address};
// ****************************************************************************

// ****************************************************************************
// ****************************************************************************
// Function exposed by the SC low level interface to the host
// CHECK: is it required? (as we use export_name)
#[no_mangle]
// specify the function name as it is seems from the outside
#[export_name = "address_receive"]
pub fn address_receive(arg_ptr: u32) -> u32 {
    let arg = get_parameters(arg_ptr);
    match Address::decode(arg.as_slice()) {
        Ok(address) => {
            log("Received address".into());
            log(format!("address: {}", address.address));
            log(format!("version: {}", address.version.to_string()));
        }
        Err(e) => {
            log(format!("Failed to decode address: {}", e));
        }
    }

    encode_length_prefixed(Vec::new())
}
