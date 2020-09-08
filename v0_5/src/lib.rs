
#![no_std]
#![allow(unused_attributes)]
#![allow(clippy::string_lit_as_bytes)]

// auxiliaries
pub mod auction_proxy;

// modules
pub mod events;
pub mod genesis;
pub mod rewards;
pub mod node_activation;
pub mod pause;
pub mod user_unstake;
pub mod user_stake;
pub mod settings;
pub mod reset_checkpoints;
pub mod reset_checkpoint_types;

use node_storage::types::*;
use crate::events::*;
use node_storage::node_config::*;
use crate::rewards::*;
use crate::node_activation::*;
use crate::pause::*;
use crate::user_stake::*;
use crate::user_unstake::*;
use user_fund_storage::user_data::*;
use user_fund_storage::fund_transf_module::*;
use user_fund_storage::fund_view_module::*;
use crate::settings::*;
use crate::reset_checkpoints::*;

imports!();

#[elrond_wasm_derive::contract(DelegationImpl)]
pub trait Delegation {

    // METADATA

    #[endpoint]
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    // MODULES

    #[module(EventsModuleImpl)]
    fn events(&self) -> EventsModuleImpl<T, BigInt, BigUint>;

    #[module(NodeConfigModuleImpl)]
    fn node_config(&self) -> NodeConfigModuleImpl<T, BigInt, BigUint>;

    #[module(RewardsModuleImpl)]
    fn rewards(&self) -> RewardsModuleImpl<T, BigInt, BigUint>;

    #[module(ResetCheckpointsModuleImpl)]
    fn reset_checkpoints(&self) -> ResetCheckpointsModuleImpl<T, BigInt, BigUint>;

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    #[module(PauseModuleImpl)]
    fn pause(&self) -> PauseModuleImpl<T, BigInt, BigUint>;

    #[module(UserStakeModuleImpl)]
    fn user_stake(&self) -> UserStakeModuleImpl<T, BigInt, BigUint>;

    #[module(NodeActivationModuleImpl)]
    fn node_activation(&self) -> NodeActivationModuleImpl<T, BigInt, BigUint>;

    #[module(UserUnStakeModuleImpl)]
    fn user_unstake(&self) -> UserUnStakeModuleImpl<T, BigInt, BigUint>;

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(FundTransformationsModuleImpl)]
    fn fund_transf_module(&self) -> FundTransformationsModuleImpl<T, BigInt, BigUint>;

    #[module(FundViewModuleImpl)]
    fn fund_view_module(&self) -> FundViewModuleImpl<T, BigInt, BigUint>;
    

    // Callbacks can only be declared here for the moment.

    #[callback]
    fn auction_stake_callback(&self,
            #[callback_arg] node_ids: Vec<usize>,
            call_result: AsyncCallResult<VarArgs<BLSStatusMultiArg>>) {

        self.node_activation().auction_stake_callback(
            node_ids,
            call_result).unwrap();
        // TODO: replace unwrap with typical Result handling
    }

    #[callback]
    fn auction_unstake_callback(&self,
            #[callback_arg] node_ids: Vec<usize>,
            call_result: AsyncCallResult<VarArgs<BLSStatusMultiArg>>) {

        self.node_activation().auction_unstake_callback(
            node_ids,
            call_result).unwrap();
            // TODO: replace unwrap with typical Result handling
    }

    #[callback]
    fn auction_unbond_callback(&self,
            #[callback_arg] node_ids: Vec<usize>,
            call_result: AsyncCallResult<VarArgs<BLSStatusMultiArg>>) {

        self.node_activation().auction_unbond_callback(
            node_ids,
            call_result).unwrap();
            // TODO: replace unwrap with typical Result handling
    }

}
