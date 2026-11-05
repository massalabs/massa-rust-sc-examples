//! An MRC20 token with the mintable, burnable and ownable extensions, entirely
//! provided by `massa-sc-standards`. The storage layout, entry-point names and
//! events are those of the AssemblyScript standard, so `massa-web3`'s `MRC20`
//! helper and any existing dApp work unchanged.
//!
//! Deploy with `constructor(name: string, symbol: string, decimals: u8, totalSupply: u256)`.
#![cfg_attr(target_arch = "wasm32", no_std)]

use massa_sc_sdk::prelude::*;
use massa_sc_standards::{export_mrc20, export_mrc20_burnable, export_mrc20_mintable, export_ownable, mrc20};

#[export]
pub fn constructor(mut args: Args) {
    let name: String = args.must_next("name");
    let symbol: String = args.must_next("symbol");
    let decimals: u8 = args.must_next("decimals");
    let total_supply: U256 = args.must_next("totalSupply");
    mrc20::constructor(&name, &symbol, decimals, total_supply);
}

// version, name, symbol, decimals, totalSupply, balanceOf, transfer, allowance,
// increaseAllowance, decreaseAllowance, transferFrom
export_mrc20!();
// mint (owner only)
export_mrc20_mintable!();
// burn, burnFrom
export_mrc20_burnable!();
// setOwner, ownerAddress, isOwner
export_ownable!();

/// A custom entry point next to the standard ones: `airdrop(recipients: string[], amount: u256)`,
/// owner only, built on the typed internals (one coin reconciliation at the end).
#[export]
pub fn airdrop(mut args: Args) {
    let initial_balance = coins::balance();
    massa_sc_standards::ownership::only_owner();
    let recipients: Vec<Address> = args.must_next("recipients");
    let amount: U256 = args.must_next("amount");
    let from = context::caller();
    for to in &recipients {
        mrc20::internals::transfer(&from, to, amount);
    }
    generate_event(&format!("airdropped {amount} to {} accounts", recipients.len()));
    coins::transfer_remaining(initial_balance, 0, None);
}

#[cfg(test)]
mod tests {
    use super::*;
    use massa_sc_sdk::mock;

    fn deploy() -> Address {
        mock::reset();
        mock::set_deploy_context(None);
        constructor(Args::new().add("Massa Token").add("MT").add(9u8).add(U256::from(1_000_000u64)));
        mock::set_admin_context(false);
        context::caller()
    }

    #[test]
    fn standard_flows() {
        let owner = deploy();
        assert_eq!(mrc20::name(), "Massa Token");
        assert_eq!(mrc20::internals::balance(&owner), U256::from(1_000_000u64));
        let bob = mock::new_user_address();
        mrc20::entrypoints::transfer(Args::new().add(&bob).add(U256::from(100u64)));
        assert_eq!(mrc20::internals::balance(&bob), U256::from(100u64));
        mrc20::mintable::entrypoints::mint(Args::new().add(&bob).add(U256::from(5u64)));
        assert_eq!(mrc20::total_supply(), U256::from(1_000_005u64));
    }

    #[test]
    fn airdrop_distributes() {
        let owner = deploy();
        let a = mock::new_user_address();
        let b = mock::new_user_address();
        airdrop(Args::new().add(vec![a.clone(), b.clone()]).add(U256::from(10u64)));
        assert_eq!(mrc20::internals::balance(&a), U256::from(10u64));
        assert_eq!(mrc20::internals::balance(&b), U256::from(10u64));
        assert_eq!(mrc20::internals::balance(&owner), U256::from(999_980u64));
    }

    #[test]
    #[should_panic(expected = "Caller is not the owner")]
    fn airdrop_is_owner_only() {
        deploy();
        mock::set_caller(&mock::new_user_address());
        airdrop(Args::new().add(Vec::<Address>::new()).add(U256::ONE));
    }
}
