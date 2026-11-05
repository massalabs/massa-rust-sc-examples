//! A persistent counter with an owner-settable step.
//!
//! Shows: typed storage, `Args` in and out, `#[derive(Serializable)]`,
//! `Result` returning entry points (an `Err` aborts with the message), owner
//! checks, and unit tests against the mock VM.
#![cfg_attr(target_arch = "wasm32", no_std)]

use massa_sc_sdk::prelude::*;

const COUNTER_KEY: &[u8] = b"counter";
const STEP_KEY: &[u8] = b"step";
const OWNER_KEY: &[u8] = b"owner";

/// Snapshot returned by `status`: a plain struct serialized field by field,
/// readable with `new Args(bytes).nextU64()...` on the client side.
#[derive(Serializable, Debug, PartialEq, Eq)]
pub struct Status {
    pub value: u64,
    pub step: u64,
    pub owner: Address,
    pub last_caller: Option<Address>,
}

/// `constructor(initial: u64 = 0, step: u64 = 1)`
#[export]
pub fn constructor(mut args: Args) {
    assert!(context::is_deploying_contract(), "constructor can only be called at deployment");
    let initial: u64 = args.next().unwrap_or(0);
    let step: u64 = args.next().unwrap_or(1);
    storage::set_value(COUNTER_KEY, &initial);
    storage::set_value(STEP_KEY, &step);
    storage::set_value(OWNER_KEY, &context::caller());
    generate_event(&format!("counter initialized to {initial} (step {step})"));
}

/// `increment(times: u64 = 1) -> u64`: add `times * step`, return the new value.
#[export]
pub fn increment(mut args: Args) -> Result<u64, &'static str> {
    let times: u64 = args.next().unwrap_or(1);
    let step: u64 = storage::get_value(STEP_KEY);
    let added = times.checked_mul(step).ok_or("overflow")?;
    let value = current().checked_add(added).ok_or("overflow")?;
    storage::set_value(COUNTER_KEY, &value);
    storage::set_value(b"last_caller", &context::caller());
    generate_event(&format!("counter is now {value}"));
    Ok(value)
}

/// `reset()`: owner only.
#[export]
pub fn reset() {
    only_owner();
    storage::set_value(COUNTER_KEY, &0u64);
    generate_event("counter reset");
}

/// `set_step(step: u64)`: owner only.
#[export]
pub fn set_step(mut args: Args) {
    only_owner();
    let step: u64 = args.must_next("step");
    assert!(step > 0, "step must be positive");
    storage::set_value(STEP_KEY, &step);
}

/// `get() -> u64`
#[export]
pub fn get() -> u64 {
    current()
}

/// `status() -> Status`
#[export]
pub fn status() -> Status {
    Status {
        value: current(),
        step: storage::get_value(STEP_KEY),
        owner: storage::get_value(OWNER_KEY),
        last_caller: storage::try_get_value(b"last_caller"),
    }
}

fn current() -> u64 {
    storage::try_get_value(COUNTER_KEY).unwrap_or(0)
}

fn only_owner() {
    let owner: Address = storage::get_value(OWNER_KEY);
    assert!(context::caller() == owner, "only the owner can do that");
}

#[cfg(test)]
mod tests {
    use super::*;
    use massa_sc_sdk::mock;

    fn deploy(initial: u64, step: u64) -> Address {
        mock::reset();
        mock::set_deploy_context(None);
        constructor(Args::new().add(initial).add(step));
        mock::set_admin_context(false);
        context::caller()
    }

    #[test]
    fn increments_by_step() {
        let owner = deploy(10, 5);
        assert_eq!(increment(Args::new()), Ok(15));
        assert_eq!(increment(Args::new().add(3u64)), Ok(30));
        assert_eq!(get(), 30);
        let s = status();
        assert_eq!(s, Status { value: 30, step: 5, owner: owner.clone(), last_caller: Some(owner) });
        // the struct round-trips through Args like any serializable value
        let bytes = Args::new().add(&s).into_bytes();
        assert_eq!(Args::from(bytes).next::<Status>().unwrap(), s);
    }

    #[test]
    fn overflow_is_an_error_not_a_wrap() {
        deploy(u64::MAX - 1, 1);
        assert_eq!(increment(Args::new().add(5u64)), Err("overflow"));
    }

    #[test]
    fn owner_only_operations() {
        deploy(0, 1);
        set_step(Args::new().add(7u64));
        assert_eq!(increment(Args::new()), Ok(7));
        reset();
        assert_eq!(get(), 0);
    }

    #[test]
    #[should_panic(expected = "only the owner")]
    fn others_cannot_reset() {
        deploy(0, 1);
        mock::set_caller(&mock::new_user_address());
        reset();
    }
}
