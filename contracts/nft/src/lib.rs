//! An MRC721 collection with the enumerable and metadata extensions, plus a
//! public `mint_next` that sells sequential token ids for a fixed MAS price.
//!
//! Deploy with `constructor(name: string, symbol: string, baseURI: string, price: u64)`.
#![cfg_attr(target_arch = "wasm32", no_std)]

use massa_sc_sdk::prelude::*;
use massa_sc_standards::mrc721::{enumerable, metadata};
use massa_sc_standards::{export_mrc721_enumerable, export_mrc721_metadata, export_ownable, ownership};

const PRICE_KEY: &[u8] = b"PRICE";
const NEXT_ID_KEY: &[u8] = b"NEXT_ID";

#[export]
pub fn constructor(mut args: Args) {
    let name: String = args.must_next("name");
    let symbol: String = args.must_next("symbol");
    let base_uri: String = args.must_next("baseURI");
    let price: u64 = args.must_next("price");
    enumerable::constructor(&name, &symbol);
    metadata::set_base_uri(&base_uri);
    storage::set_value(PRICE_KEY, &price);
    storage::set_value(NEXT_ID_KEY, &1u64);
}

// name, symbol, version, balanceOf, ownerOf, getApproved, isApprovedForAll, approve,
// setApprovalForAll, ownerAddress, totalSupply, transferFrom, mint (owner), burn
export_mrc721_enumerable!();
// uri(tokenId)
export_mrc721_metadata!();
export_ownable!();

/// `mint_next() -> u256`: anyone can mint the next token id by sending `price`
/// nanoMAS (plus the storage cost of the new entries; the surplus is refunded).
#[export]
pub fn mint_next() -> U256 {
    let initial_balance = coins::balance();
    let price: u64 = storage::get_value(PRICE_KEY);
    let id_num: u64 = storage::get_value(NEXT_ID_KEY);
    let id = U256::from(id_num);
    let buyer = context::caller();
    enumerable::update(Some(&buyer), id, None);
    metadata::set_uri(id, &format!("{id_num}.json"));
    storage::set_value(NEXT_ID_KEY, &(id_num + 1));
    generate_event(&format!("minted #{id_num} for {buyer}"));
    // keep `price` for the contract, refund the rest to the buyer
    coins::transfer_remaining(initial_balance, price, None);
    id
}

/// `set_price(price: u64)`: owner only.
#[export]
pub fn set_price(mut args: Args) {
    ownership::only_owner();
    let price: u64 = args.must_next("price");
    storage::set_value(PRICE_KEY, &price);
}

/// `withdraw(to: string)`: owner only, sends the contract balance to `to`.
#[export]
pub fn withdraw(mut args: Args) {
    ownership::only_owner();
    let to: Address = args.must_next("to");
    coins::transfer_coins(&to, coins::balance());
}

/// `tokens_of(owner: string) -> u256[]`
///
/// Uses the legacy key query so the contract also runs on nodes without
/// MIP-0002 (C-Layer); `enumerable::owned_tokens_page` paginates on Massa.
#[export]
pub fn tokens_of(mut args: Args) -> Args {
    let owner: Address = args.must_next("owner");
    Args::new().add(enumerable::owned_tokens(&owner))
}

#[cfg(test)]
mod tests {
    use super::*;
    use massa_sc_sdk::mock;

    fn deploy() -> Address {
        mock::reset();
        mock::set_deploy_context(None);
        constructor(Args::new().add("Massa Punks").add("PUNK").add("ipfs://base/").add(1_000_000u64));
        mock::set_admin_context(false);
        context::caller()
    }

    #[test]
    fn public_mint_charges_the_price() {
        let owner = deploy();
        let buyer = mock::new_user_address();
        mock::set_caller(&buyer);
        mock::set_balance(&buyer, 0);
        let contract_balance = coins::balance();
        mock::set_transferred_coins(1_500_000);
        mock::set_balance(&context::callee(), contract_balance + 1_500_000);
        let id = mint_next();
        assert_eq!(id, U256::ONE);
        assert_eq!(enumerable::owner_of(U256::ONE), Some(buyer.clone()));
        assert_eq!(metadata::uri(U256::ONE), "ipfs://base/1.json");
        assert_eq!(enumerable::total_supply(), U256::ONE);
        // 1.5 MAS sent, 1 MAS kept, 0.5 MAS refunded
        assert_eq!(mock::balance_of(&buyer), 500_000);
        let mut r = tokens_of(Args::new().add(&buyer));
        assert_eq!(r.next::<Vec<U256>>().unwrap(), vec![U256::ONE]);
        assert!(ownership::is_owner(&owner));
    }

    #[test]
    #[should_panic(expected = "INSUFFICIENT_COINS_TRANSFERRED")]
    fn mint_requires_payment() {
        deploy();
        mock::set_transferred_coins(10);
        mint_next();
    }
}
