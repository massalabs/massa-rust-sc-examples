//! Multi-contract deployer, a port of the standard AssemblyScript deployer
//! used by `massa-web3`. It runs as an `ExecuteSC` operation whose datastore
//! carries, for `i` in `1..=count`:
//!
//! | key (Args encoding)   | value                                   |
//! |-----------------------|-----------------------------------------|
//! | `[0]`                 | `count: u64`                            |
//! | `u64(i)`              | bytecode of contract `i`                |
//! | `u64(i) ++ bytes([0])`| optional constructor arguments          |
//! | `u64(i) ++ bytes([1])`| optional coins sent to the constructor  |
//!
//! Each contract is created, its `constructor` is called when it exists, and
//! the deployed addresses are reported in events (one per contract, plus a
//! base64 raw event with the serialized address list).
#![cfg_attr(target_arch = "wasm32", no_std)]

use massa_sc_sdk::prelude::*;

const CONSTRUCTOR: &str = "constructor";

#[export]
pub fn main() {
    let count = contract_count();
    let mut deployed: Vec<Address> = Vec::with_capacity(count as usize);
    for i in 0..count {
        let address = contract::create_sc(&bytecode(i));
        if contract::function_exists(&address, CONSTRUCTOR) {
            contract::call(&address, CONSTRUCTOR, constructor_args(i), coins(i));
        }
        generate_event(&format!("Contract deployed at address: {address}"));
        deployed.push(address);
    }
    generate_raw_event(Args::new().add(&deployed).as_bytes());
}

fn contract_count() -> u64 {
    assert!(op_datastore::has_op_key([0u8]), "The number of smart contracts to deploy is undefined.");
    Args::from(op_datastore::get_op_data([0u8])).must_next("nbSC")
}

fn bytecode(i: u64) -> Vec<u8> {
    let key = Args::new().add(i + 1);
    assert!(op_datastore::has_op_key(&key), "No bytecode found for contract number: {}", i + 1);
    op_datastore::get_op_data(&key)
}

fn constructor_args(i: u64) -> Vec<u8> {
    let key = Args::new().add(i + 1).add([0u8]);
    op_datastore::try_get_op_data(&key).unwrap_or_default()
}

fn coins(i: u64) -> u64 {
    let key = Args::new().add(i + 1).add([1u8]);
    op_datastore::try_get_op_data(&key).and_then(|v| Args::from(v).next_u64().ok()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use massa_sc_sdk::mock;

    #[test]
    fn deploys_two_contracts() {
        mock::reset();
        mock::set_local_context(None);
        mock::set_op_datastore_entry([0u8], Args::new().add(2u64));
        mock::set_op_datastore_entry(Args::new().add(1u64), b"\0asm-one".to_vec());
        mock::set_op_datastore_entry(Args::new().add(1u64).add([0u8]), Args::new().add("init"));
        mock::set_op_datastore_entry(Args::new().add(1u64).add([1u8]), Args::new().add(5_000u64));
        mock::set_op_datastore_entry(Args::new().add(2u64), b"\0asm-two".to_vec());
        mock::set_call_handler(|_, function, args, coins| {
            assert_eq!(function, "constructor");
            if coins == 5_000 {
                assert_eq!(args, Args::new().add("init").as_bytes());
            } else {
                assert!(args.is_empty());
            }
            Some(Vec::new())
        });
        main();
        let events = mock::events();
        assert_eq!(events.len(), 3);
        assert!(events[0].starts_with("Contract deployed at address: AS"));
        let addresses: Vec<Address> =
            Args::from(massa_sc_sdk::events::base64_decode(&events[2]).unwrap()).next().unwrap();
        assert_eq!(addresses.len(), 2);
        assert_eq!(contract::get_bytecode_of(&addresses[1]), b"\0asm-two");
        assert_eq!(mock::recorded_calls().len(), 2);
    }

    #[test]
    #[should_panic(expected = "undefined")]
    fn requires_a_count() {
        mock::reset();
        mock::set_local_context(None);
        main();
    }
}
