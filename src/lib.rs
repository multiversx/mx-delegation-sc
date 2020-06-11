
#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![allow(unused_attributes)]

// auxiliaries
pub mod auction_proxy;
pub mod bls_key;
pub mod node_state;
pub mod unbond_queue;
pub mod user_stake_state;
pub mod util;

use crate::unbond_queue::*;

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

use crate::events::*;
use crate::node_config::*;
use crate::rewards::*;
use crate::node_activation::*;
use crate::pause::*;
use crate::user_stake::*;
use crate::stake_sale::*;
use crate::unexpected::*;
use crate::user_data::*;
use crate::settings::*;

// increment this whenever changing the contract
const VERSION: &[u8] = b"0.3.3";

imports!();

#[elrond_wasm_derive::contract(DelegationImpl)]
pub trait Delegation {

    // METADATA

    fn version(&self) -> Vec<u8> {
        VERSION.to_vec()
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
    

    // Callbacks can only be declared here for the moment.

    #[callback]
    fn auction_stake_callback(&self,
            #[callback_arg] node_ids: Vec<usize>,
            call_result: AsyncCallResult<VarArgs<Vec<u8>>>) {

        self.node_activation().auction_stake_callback(
            node_ids,
            call_result).unwrap();
        // TODO: replace unwrap with typical Result handling
    }

    #[callback]
    fn auction_unStake_callback(&self,
            #[callback_arg] opt_unbond_queue_entry: Option<UnbondQueueItem<BigUint>>,
            #[callback_arg] node_ids: Vec<usize>,
            call_result: AsyncCallResult<VarArgs<Vec<u8>>>) {

        self.node_activation().auction_unStake_callback(
            opt_unbond_queue_entry,
            node_ids,
            call_result).unwrap();
            // TODO: replace unwrap with typical Result handling
    }

    #[callback]
    fn auction_unBond_callback(&self,
            #[callback_arg] node_ids: Vec<usize>,
            call_result: AsyncCallResult<VarArgs<Vec<u8>>>) {

        self.node_activation().auction_unBond_callback(
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
