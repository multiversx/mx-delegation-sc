
#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![allow(unused_attributes)]

// auxiliaries
pub mod auction_proxy;
pub mod bls_key;
pub mod node_state;
pub mod util;

// modules
pub mod events;
// pub mod genesis;
pub mod nodes;
pub mod rewards;
pub mod stake_per_node;
pub mod stake_per_user;
pub mod stake_sale;
// pub mod unexpected;
pub mod user_data;
pub mod settings;

use crate::events::*;
use crate::nodes::*;
use crate::rewards::*;
use crate::stake_per_node::*;
use crate::stake_per_user::*;
use crate::stake_sale::*;
// use crate::unexpected::*;
use crate::user_data::*;
use crate::settings::*;

// increment this whenever changing the contract
const VERSION: &[u8] = b"0.3.0";

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

    #[module(NodeModuleImpl)]
    fn nodes(&self) -> NodeModuleImpl<T, BigInt, BigUint>;

    #[module(RewardsModuleImpl)]
    fn rewards(&self) -> RewardsModuleImpl<T, BigInt, BigUint>;

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    #[module(UserStakeModuleImpl)]
    fn user_stake(&self) -> UserStakeModuleImpl<T, BigInt, BigUint>;

    #[module(ContractStakeModuleImpl)]
    fn contract_stake(&self) -> ContractStakeModuleImpl<T, BigInt, BigUint>;

    #[module(StakeSaleModuleImpl)]
    fn stake_sale(&self) -> StakeSaleModuleImpl<T, BigInt, BigUint>;

    // #[module(UnexpectedBalanceModuleImpl)]
    // fn unexpected(&self) -> UnexpectedBalanceModuleImpl<T, BigInt, BigUint>;

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;
    

    // Callbacks can only be declared here for the moment.

    #[callback]
    fn auction_stake_callback(&self,
            #[callback_arg] node_ids: Vec<usize>,
            call_result: AsyncCallResult<()>) {

        let _ = self.contract_stake().auction_stake_callback(
            node_ids,
            call_result);
    }

    #[callback]
    fn auction_unStake_callback(&self,
            #[callback_arg] node_ids: Vec<usize>,
            call_result: AsyncCallResult<()>) {

        let _ = self.contract_stake().auction_unStake_callback(
            node_ids,
            call_result);
    }

    #[callback]
    fn auction_unBond_callback(&self,
            #[callback_arg] node_ids: Vec<usize>,
            call_result: AsyncCallResult<()>) {

        let _ = self.contract_stake().auction_unBond_callback(
            node_ids,
            call_result);
    }
    
}
