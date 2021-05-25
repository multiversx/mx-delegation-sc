#![no_std]
#![allow(unused_attributes)]
#![allow(unused_imports)] // TODO: for now
#![allow(clippy::string_lit_as_bytes)]
#![allow(non_snake_case)] // until we sort out call proxy endpoint names

// auxiliaries
pub mod auction_proxy;

// modules
pub mod events;
pub mod node_activation;
pub mod reset_checkpoint_endpoints;
pub mod reset_checkpoint_state;
pub mod reset_checkpoint_types;
pub mod rewards_endpoints;
pub mod rewards_state;
pub mod settings;
pub mod user_stake_endpoints;
pub mod user_stake_state;

#[cfg(feature = "node-storage-default")]
pub use node_storage_default as node_storage;
#[cfg(feature = "node-storage-wasm")]
pub use node_storage_wasm as node_storage;

#[cfg(feature = "user-fund-storage-default")]
pub use user_fund_storage_default as user_fund_storage;
#[cfg(feature = "user-fund-storage-wasm")]
pub use user_fund_storage_wasm as user_fund_storage;

#[cfg(feature = "elrond-wasm-module-features-default")]
pub use elrond_wasm_module_features_default as elrond_wasm_module_features;
#[cfg(feature = "elrond-wasm-module-features-wasm")]
pub use elrond_wasm_module_features_wasm as elrond_wasm_module_features;

#[cfg(feature = "elrond-wasm-module-pause-default")]
pub use elrond_wasm_module_pause_default as elrond_wasm_module_pause;
#[cfg(feature = "elrond-wasm-module-pause-wasm")]
pub use elrond_wasm_module_pause_wasm as elrond_wasm_module_pause;

pub use elrond_wasm_module_pause::*;
pub use node_storage::node_config::*;
pub use node_storage::types::*;
pub use user_fund_storage::fund_transf_module::*;
pub use user_fund_storage::fund_view_module::*;
pub use user_fund_storage::types::*;
pub use user_fund_storage::user_data::*;
