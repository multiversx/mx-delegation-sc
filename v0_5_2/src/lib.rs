
#![no_std]
#![allow(unused_attributes)]
#![allow(clippy::string_lit_as_bytes)]

// auxiliaries
pub mod auction_proxy;

// modules
pub mod events;
pub mod rewards;
pub mod node_activation;
pub mod user_unstake;
pub mod user_stake;
pub mod settings;
pub mod reset_checkpoints;
pub mod reset_checkpoint_types;

#[macro_use]
extern crate elrond_wasm;

pub use crate::events::*;
pub use crate::rewards::*;
pub use crate::node_activation::*;
pub use crate::user_stake::*;
pub use crate::user_unstake::*;
pub use crate::settings::*;
pub use crate::reset_checkpoints::*;

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

pub use node_storage::types::*;
pub use node_storage::node_config::*;
pub use user_fund_storage::user_data::*;
pub use user_fund_storage::fund_transf_module::*;
pub use user_fund_storage::fund_view_module::*;
pub use user_fund_storage::types::*;
pub use elrond_wasm_module_pause::*;
