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
use sdk::abis::generate_event;
// ****************************************************************************

// ****************************************************************************
// Simple Smart Contract that generate an event
// ****************************************************************************
#[no_mangle]
pub extern "C" fn call_generate_event(_arg: i32) -> i32 {
    let event = "I'am a Smart Contract written in Rust".to_string();
    generate_event(event);
    return 0;
}

#[cfg(test)]
mod tests {
    use crate::call_generate_event;

    #[test]
    fn test_main() {
        assert_eq!(0, call_generate_event(0));
    }
}
