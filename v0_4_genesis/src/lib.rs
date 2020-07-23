
#![no_std]
#![no_main]
#![allow(unused_attributes)]
#![allow(clippy::string_lit_as_bytes)]

// auxiliaries
pub mod types;

// modules
pub mod events;
pub mod genesis;
pub mod node_config;
pub mod rewards;
pub mod user_data;
pub mod user_stake;
pub mod settings;
pub mod fund_module;
pub mod fund_transf_module;
pub mod fund_view_module;

imports!();

#[elrond_wasm_derive::contract(DelegationImpl)]
pub trait Delegation {

    // METADATA

    #[endpoint]
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    
}
