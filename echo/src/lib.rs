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
use sdk::abis::{echo, log};
// ****************************************************************************

// ****************************************************************************
// SC can exposte a main function, but it is not required and it must be
// preceded by #[cfg(not(test))] to avoid conflicts with the test runner
// ****************************************************************************
// #[cfg(not(test))]
// #[no_mangle]
// pub extern "C" fn main(_arg: i32) -> i32 {
//     panic!("end");
// }

// ****************************************************************************
// Function exposed by the SC low level interface to the host
// CHECK: is it required? (as we use export_name)
#[no_mangle]
// specify the function name as it is seems from the outside
#[export_name = "call_echo"]
pub fn call_echo(arg_ptr: u32) -> u32 {
    let arg = get_parameters(arg_ptr);
    log("call_echo".to_string());

    let ret = echo(arg.clone());
    assert_eq!(ret, arg);

    // data MUST be returned this way
    encode_length_prefixed(ret)
}

#[cfg(test)]
mod tests {
    // *************************************************************************
    // Imports from the std crate to return to std environment and write tests
    // more simply
    // *************************************************************************
    extern crate std;
    use massa_rust_sc_sdk as sdk;
    use sdk::{test, *};

    use crate::call_echo;

    #[test]
    fn test_call_echo() {
        let test_msg = "test".to_string().into_bytes();

        // simulate arguments passing from the host to the SC
        let buf_ptr = test::host_write_buffer(&test_msg);

        // call the SC function and get the result
        let result_ptr = call_echo(buf_ptr);

        // simulate reading the result from the SC
        let result = test::host_read_buffer(result_ptr);

        // decode the result from the SC
        let result = String::from_utf8_lossy(&result);

        assert_eq!(result, "test");
    }

    #[test]
    #[should_panic]
    fn test_panic() {
        panic!("test panic");
    }
}
