
#![no_std]
#![no_main]
#![allow(unused_attributes)]
#![allow(clippy::string_lit_as_bytes)]

// auxiliaries
pub mod auction_proxy;
pub mod bls_key;
pub mod node_state;
pub mod stake_sale_payment;
pub mod unbond_queue;
pub mod user_stake_state;
pub mod util;
pub mod fund_item;
pub mod fund_list;
pub mod fund_type;

// use crate::unbond_queue::*;
use crate::bls_key::*;

// modules
pub mod events;
pub mod genesis;
pub mod node_config;
pub mod rewards;
pub mod node_activation;
pub mod pause;
pub mod stake_sale;
pub mod unexpected;
pub mod user_data;
pub mod user_stake;
pub mod settings;
pub mod fund_module;
pub mod fund_transf_module;
pub mod fund_view_module;

use crate::events::*;
use crate::node_config::*;
use crate::rewards::*;
use crate::node_activation::*;
use crate::pause::*;
use crate::user_stake::*;
use crate::stake_sale::*;
use crate::unexpected::*;
use crate::user_data::*;
use crate::fund_transf_module::*;
use crate::fund_view_module::*;
use crate::settings::*;

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

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    #[module(PauseModuleImpl)]
    fn pause(&self) -> PauseModuleImpl<T, BigInt, BigUint>;

    #[module(UserStakeModuleImpl)]
    fn user_stake(&self) -> UserStakeModuleImpl<T, BigInt, BigUint>;

    #[module(NodeActivationModuleImpl)]
    fn node_activation(&self) -> NodeActivationModuleImpl<T, BigInt, BigUint>;

    #[module(StakeSaleModuleImpl)]
    fn stake_sale(&self) -> StakeSaleModuleImpl<T, BigInt, BigUint>;

    #[module(UnexpectedBalanceModuleImpl)]
    fn unexpected(&self) -> UnexpectedBalanceModuleImpl<T, BigInt, BigUint>;

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

    #[callback]
    fn auction_claim_callback(&self,
            #[callback_arg] node_ids: Vec<usize>,
            call_result: AsyncCallResult<()>) {

        self.node_activation().auction_claim_callback(
            node_ids,
            call_result).unwrap();
            // TODO: replace unwrap with typical Result handling
    }
    
}
