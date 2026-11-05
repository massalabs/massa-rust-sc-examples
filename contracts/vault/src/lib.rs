//! Cross-contract interactions:
//!
//! * `deposit`/`withdraw` move MRC20 tokens between users and the vault with
//!   the [`Mrc20`] client (`transferFrom` needs a prior allowance to the vault),
//!   covering the token's storage cost with [`Mrc20::balance_entry_cost`];
//! * `spawn_counter` deploys an embedded child contract (the `counter`
//!   example, included at build time) and initializes it, like the deployer;
//! * `peek_counter` shows `local_call`: running the child's `get` in the
//!   vault's context.
//!
//! Deploy with `constructor(token: string)`.
#![cfg_attr(target_arch = "wasm32", no_std)]

use massa_sc_sdk::collections::PersistentMap;
use massa_sc_sdk::prelude::*;
use massa_sc_standards::mrc20::Mrc20;

const TOKEN_KEY: &[u8] = b"TOKEN";
const CHILD_KEY: &[u8] = b"CHILD";

/// Bytecode of the `counter` example, built first by `cargo massa build`
/// (see `build.rs`).
const COUNTER_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/counter.wasm"));

fn balances() -> PersistentMap<Address, U256> {
    PersistentMap::new("bal")
}

fn token() -> Mrc20 {
    Mrc20::new(storage::get_value(TOKEN_KEY))
}

#[export]
pub fn constructor(mut args: Args) {
    assert!(context::is_deploying_contract(), "constructor can only be called at deployment");
    let token: Address = args.must_next("token");
    contract::assert_is_smart_contract(&token);
    storage::set_value(TOKEN_KEY, &token);
}

/// `deposit(amount: u256)`: pulls `amount` tokens from the caller (who must
/// have approved the vault) and credits its vault balance. Send enough coins
/// to pay the storage of the caller's vault entry and, the first time, of the
/// vault's balance entry in the token.
#[export]
pub fn deposit(mut args: Args) {
    let initial_balance = coins::balance();
    let amount: U256 = args.must_next("amount");
    let user = context::caller();
    let vault = context::callee();
    let token = token();
    token.transfer_from(&user, &vault, amount, token.balance_entry_cost(&vault));
    let map = balances();
    let new = map.get_or_default(&user).checked_add(amount).expect("overflow");
    map.set(&user, new);
    generate_event(&format!("deposit:{user},{amount}"));
    coins::transfer_remaining(initial_balance, 0, None);
}

/// `withdraw(amount: u256)`: sends tokens back to the caller.
#[export]
pub fn withdraw(mut args: Args) {
    let initial_balance = coins::balance();
    let amount: U256 = args.must_next("amount");
    let user = context::caller();
    let map = balances();
    let current = map.get_or_default(&user);
    assert!(current >= amount, "insufficient vault balance");
    if current == amount {
        map.delete(&user);
    } else {
        map.set(&user, current - amount);
    }
    let token = token();
    token.transfer(&user, amount, token.balance_entry_cost(&user));
    generate_event(&format!("withdraw:{user},{amount}"));
    coins::transfer_remaining(initial_balance, 0, None);
}

/// `balance_of(user: string) -> u256`
#[export]
pub fn balance_of(mut args: Args) -> U256 {
    let user: Address = args.must_next("user");
    balances().get_or_default(&user)
}

/// `spawn_counter(initial: u64) -> string`: deploys the embedded counter,
/// calls its constructor and remembers it. The coins sent with the call pay
/// the child's storage ([`contract::deploy_cost`], about 2 MAS on Massa);
/// whatever remains is given to the child.
#[export]
pub fn spawn_counter(mut args: Args) -> Address {
    let initial: u64 = args.next().unwrap_or(0);
    let cost = contract::deploy_cost(COUNTER_WASM.len());
    let sent = context::transferred_coins();
    assert!(sent >= cost, "send at least {cost} nanoMAS to pay the child's storage");
    let child = contract::create_sc(COUNTER_WASM);
    // The vault has write access to the child during this execution only.
    if sent > cost {
        coins::transfer_coins(&child, sent - cost);
    }
    contract::call(&child, "constructor", Args::new().add(initial).add(1u64), 0);
    storage::set_value(CHILD_KEY, &child);
    generate_event(&format!("spawned counter at {child}"));
    child
}

/// `bump_counter() -> u64`: calls the child's `increment` (regular call).
#[export]
pub fn bump_counter() -> u64 {
    let child: Address = storage::get_value(CHILD_KEY);
    let ret = contract::call(&child, "increment", Args::new(), 0);
    u64::decode_bytes(&ret).expect("u64")
}

/// `peek_counter() -> u64`: runs the child's `get` *in the vault's context*
/// (it reads the vault's datastore, where no counter lives, hence 0): a
/// demonstration of what `local_call` does, not something you usually want.
#[export]
pub fn peek_counter() -> u64 {
    let child: Address = storage::get_value(CHILD_KEY);
    let ret = contract::local_call(&child, "get", Args::new());
    u64::decode_bytes(&ret).expect("u64")
}

#[cfg(test)]
mod tests {
    use super::*;
    use massa_sc_sdk::mock;

    fn deploy() -> (Address, Address) {
        mock::reset();
        let token = mock::new_sc_address();
        mock::set_bytecode_of(&token, b"\0asm");
        mock::set_deploy_context(None);
        constructor(Args::new().add(&token));
        mock::set_admin_context(false);
        (token, context::caller())
    }

    #[test]
    fn deposit_and_withdraw_through_the_token() {
        let (token, user) = deploy();
        let vault = context::callee();
        // The mocked token accepts every call.
        mock::set_call_handler(|_, _, _, _| Some(Vec::new()));
        // The user sends 1 MAS with each call to cover the token's storage
        // costs; whatever is not needed comes back.
        let send = |amount: u64| {
            mock::set_transferred_coins(amount);
            mock::set_balance(&vault, mock::balance_of(&vault) + amount);
        };
        mock::set_balance(&user, 0);
        send(coins::ONE_MAS);
        deposit(Args::new().add(U256::from(50u64)));
        assert_eq!(balance_of(Args::new().add(&user)), U256::from(50u64));
        let entry_cost = Mrc20::new(token.clone()).balance_entry_cost(&vault);
        assert_eq!(mock::balance_of(&user), coins::ONE_MAS - entry_cost, "surplus refunded");

        send(coins::ONE_MAS);
        withdraw(Args::new().add(U256::from(20u64)));
        assert_eq!(balance_of(Args::new().add(&user)), U256::from(30u64));
        let calls = mock::recorded_calls();
        assert_eq!(calls[0].address, token.as_str());
        assert_eq!(calls[0].function, "transferFrom");
        assert_eq!(calls[0].coins, entry_cost);
        assert_eq!(calls[1].function, "transfer");
        assert_eq!(calls[1].args, Args::new().add(&user).add(U256::from(20u64)).into_bytes());
    }

    #[test]
    fn spawns_and_calls_a_child() {
        deploy();
        mock::set_call_handler(|_, function, _, _| match function {
            "constructor" => Some(Vec::new()),
            "increment" => Some(1u64.to_le_bytes().to_vec()),
            "get" => Some(0u64.to_le_bytes().to_vec()),
            _ => None,
        });
        // Not enough coins to pay the child's storage: rejected.
        mock::set_transferred_coins(1);
        assert!(std::panic::catch_unwind(|| spawn_counter(Args::new().add(0u64))).is_err());
        let cost = contract::deploy_cost(COUNTER_WASM.len());
        let vault = context::callee();
        mock::set_transferred_coins(cost + 7);
        mock::set_balance(&vault, mock::balance_of(&vault) + cost + 7);
        let child = spawn_counter(Args::new().add(0u64));
        assert!(child.as_str().starts_with("AS"));
        assert_eq!(mock::balance_of(&child), 7, "surplus forwarded to the child");
        assert_eq!(contract::get_bytecode_of(&child), COUNTER_WASM);
        assert_eq!(bump_counter(), 1);
        assert_eq!(peek_counter(), 0);
        assert_eq!(mock::recorded_calls()[0].function, "constructor");
        assert!(!mock::recorded_calls()[2].remote);
    }

    #[test]
    #[should_panic(expected = "insufficient vault balance")]
    fn cannot_overdraw() {
        deploy();
        withdraw(Args::new().add(U256::ONE));
    }
}
