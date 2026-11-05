//! An autonomous smart contract: once started, `tick` books its own next
//! execution with a deferred call, every `period_gap` periods, until `stop`
//! is called or the contract runs out of coins. Also shows the legacy
//! asynchronous message API.
//!
//! Deploy with `constructor()`, fund the contract with a few MAS (booking a
//! deferred call costs coins), then call `start(period_gap: u64 = 10)`.
#![cfg_attr(target_arch = "wasm32", no_std)]

use massa_sc_sdk::prelude::*;

const TICKS_KEY: &[u8] = b"ticks";
const GAP_KEY: &[u8] = b"gap";
const NEXT_CALL_KEY: &[u8] = b"next_call";
const OWNER_KEY: &[u8] = b"owner";

/// Gas booked for each `tick` execution.
const TICK_GAS: u64 = 10_000_000;

#[export]
pub fn constructor(_args: Args) {
    assert!(context::is_deploying_contract(), "constructor can only be called at deployment");
    storage::set_value(TICKS_KEY, &0u64);
    storage::set_value(OWNER_KEY, &context::caller());
}

/// `start(period_gap: u64 = 10)`: schedule the first tick (owner only).
#[export]
pub fn start(mut args: Args) {
    only_owner();
    let gap: u64 = args.next().unwrap_or(10);
    assert!(gap >= 1, "period gap must be at least 1");
    storage::set_value(GAP_KEY, &gap);
    schedule_next_tick();
}

/// `stop()`: cancel the pending tick (owner only). The booked coins are refunded.
#[export]
pub fn stop() {
    only_owner();
    if let Some(id) = storage::try_get_value::<String>(NEXT_CALL_KEY) {
        if deferred_calls::exists(&id) {
            deferred_calls::cancel(&id);
        }
        storage::del(NEXT_CALL_KEY);
        generate_event("stopped");
    }
}

/// `tick()`: the scheduled work. Only the contract itself may call it (the
/// deferred call executes with the contract as caller).
#[export]
pub fn tick() {
    assert!(context::caller() == context::callee(), "tick can only be triggered by the scheduled call");
    let ticks: u64 = storage::get_value::<u64>(TICKS_KEY) + 1;
    storage::set_value(TICKS_KEY, &ticks);
    let slot = context::current_slot();
    generate_event(&format!("tick #{ticks} at slot {}/{}", slot.period, slot.thread));
    if storage::has(NEXT_CALL_KEY) {
        schedule_next_tick();
    }
}

/// `ticks() -> u64`
#[export]
pub fn ticks() -> u64 {
    storage::get_value(TICKS_KEY)
}

/// `next_call_id() -> string` (empty when nothing is scheduled)
#[export]
pub fn next_call_id() -> String {
    storage::try_get_value(NEXT_CALL_KEY).unwrap_or_default()
}

/// Books `tick` in the cheapest slot of the target period and stores the id.
fn schedule_next_tick() {
    let gap: u64 = storage::get_value(GAP_KEY);
    let target_period = context::current_period() + gap;
    let slot = deferred_calls::find_cheapest_slot(target_period, target_period, TICK_GAS, 0)
        .expect("no slot available in the target period");
    let price = deferred_calls::quote(slot, TICK_GAS, 0);
    assert!(coins::balance() >= price, "not enough coins to book the next tick");
    let id = deferred_calls::register(&context::callee(), "tick", slot, TICK_GAS, [], 0);
    storage::set_value(NEXT_CALL_KEY, &id);
    generate_event(&format!(
        "next tick booked in slot {}/{} for {price} nanoMAS ({id})",
        slot.period, slot.thread
    ));
}

/// `send_legacy_message(target: string, function: string)`: the older
/// "asynchronous message" mechanism, executed as soon as possible between the
/// next period and 10 periods later. Owner only.
#[export]
pub fn send_legacy_message(mut args: Args) {
    only_owner();
    let target: Address = args.must_next("target");
    let function: String = args.must_next("function");
    let start = Slot::new(context::current_period() + 1, 0);
    let end = Slot::new(context::current_period() + 10, 31);
    contract::async_call(&target, &function, start, end, TICK_GAS, 1_000_000, 0, Args::new(), None);
    generate_event(&format!("async message sent to {target}::{function}"));
}

fn only_owner() {
    assert!(context::caller() == storage::get_value::<Address>(OWNER_KEY), "only the owner can do that");
}

#[cfg(test)]
mod tests {
    use super::*;
    use massa_sc_sdk::mock;

    fn deploy() {
        mock::reset();
        mock::set_deploy_context(None);
        constructor(Args::new());
        mock::set_admin_context(false);
        mock::set_slot(100, 5);
    }

    #[test]
    fn schedules_itself() {
        deploy();
        start(Args::new().add(3u64));
        let booked = mock::deferred_calls();
        assert_eq!(booked.len(), 1);
        assert_eq!(booked[0].slot.0, 103);
        assert_eq!(booked[0].function, "tick");
        assert_eq!(next_call_id(), booked[0].id);

        // the deferred call runs with the contract as caller
        mock::set_slot(103, booked[0].slot.1);
        let me = context::callee();
        mock::set_call_stack(&[me.as_str(), me.as_str()]);
        tick();
        assert_eq!(ticks(), 1);
        assert_eq!(mock::deferred_calls().len(), 2, "rescheduled");
    }

    #[test]
    fn stop_cancels() {
        deploy();
        start(Args::new());
        let id = next_call_id();
        assert!(deferred_calls::exists(&id));
        stop();
        assert!(!deferred_calls::exists(&id));
        assert_eq!(next_call_id(), "");
    }

    #[test]
    #[should_panic(expected = "scheduled call")]
    fn tick_is_protected() {
        deploy();
        tick();
    }

    #[test]
    fn legacy_message() {
        deploy();
        let target = mock::new_sc_address();
        send_legacy_message(Args::new().add(&target).add("wake_up"));
        let msgs = mock::async_messages();
        assert_eq!(msgs[0].address, target.as_str());
        assert_eq!(msgs[0].validity_start, (101, 0));
    }
}
