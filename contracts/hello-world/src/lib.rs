//! The smallest useful Massa contract.
//!
//! * `main` runs when the module is sent as an `ExecuteSC` operation (no
//!   deployment, nothing persists, only the events).
//! * `hello` is a regular entry point callable once the contract is deployed
//!   (deploy with an empty constructor argument list).
#![cfg_attr(target_arch = "wasm32", no_std)]

use massa_sc_sdk::prelude::*;

/// Entry point of an `ExecuteSC` operation.
#[export]
pub fn main() {
    generate_event(&format!(
        "Hello from an ExecuteSC run by {} at period {}",
        context::transaction_creator(),
        context::current_period()
    ));
}

/// Deployment hook called by the deployer; nothing to initialize here.
#[export]
pub fn constructor(_args: Args) {
    assert!(context::is_deploying_contract(), "constructor can only be called at deployment");
    generate_event("hello-world deployed");
}

/// `hello(name: string) -> string`
#[export]
pub fn hello(mut args: Args) -> String {
    let name: String = args.next().unwrap_or_else(|_| "world".to_string());
    let message = format!("Hello, {name}!");
    generate_event(&message);
    message
}

#[cfg(test)]
mod tests {
    use super::*;
    use massa_sc_sdk::mock;

    #[test]
    fn greets() {
        mock::reset();
        assert_eq!(hello(Args::new().add("Massa")), "Hello, Massa!");
        assert_eq!(hello(Args::new()), "Hello, world!");
        assert_eq!(mock::events(), vec!["Hello, Massa!", "Hello, world!"]);
    }

    #[test]
    fn main_emits_an_event() {
        mock::reset();
        mock::set_local_context(None);
        mock::set_slot(42, 0);
        main();
        assert!(mock::events()[0].contains("period 42"));
    }
}
