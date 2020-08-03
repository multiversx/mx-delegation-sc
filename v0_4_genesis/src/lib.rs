
#![no_std]
#![no_main]
#![allow(unused_attributes)]
#![allow(clippy::string_lit_as_bytes)]

// modules
pub mod events;
pub mod genesis;
pub mod rewards;
pub mod user_stake;
pub mod settings;

imports!();

#[elrond_wasm_derive::contract(DelegationImpl)]
pub trait Delegation {

    // METADATA

    #[endpoint]
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    
}
