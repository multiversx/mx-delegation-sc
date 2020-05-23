
#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![allow(unused_attributes)]

// pub mod auction_proxy;
pub mod bls_key;
pub mod stake_state;
pub mod util;

// use auction_proxy::Auction;
use crate::bls_key::*;

// modules
pub mod events;
pub mod genesis;
pub mod nodes;
pub mod rewards;
pub mod stake_per_contract;
pub mod stake_per_user;
pub mod stake_sale;
pub mod unexpected;
pub mod user_data;
pub mod settings;

use crate::events::*;
use crate::nodes::*;
use crate::rewards::*;
use crate::stake_per_contract::*;
use crate::stake_per_user::*;
use crate::stake_sale::*;
use crate::unexpected::*;
use crate::user_data::*;
use crate::settings::*;

#[elrond_wasm_derive::callable(AuctionProxy)]
pub trait Auction {
    #[payable]
    #[callback(auction_stake_callback)]
    fn stake(&self,
        num_nodes: usize,
        #[multi(2*num_nodes)] bls_keys_signatures: Vec<Vec<u8>>,
        #[payment] payment: &BigUint);

    #[callback(auction_unStake_callback)]
    fn unStake(&self,
        #[var_args] bls_keys_signatures: Vec<BLSKey>);

    #[callback(auction_unBond_callback)]
    fn unBond(&self,
        #[var_args] bls_keys_signatures: Vec<BLSKey>);
}

// increment this whenever changing the contract
const VERSION: &[u8] = b"0.2.2";

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

    #[module(UnexpectedBalanceModuleImpl)]
    fn unexpected(&self) -> UnexpectedBalanceModuleImpl<T, BigInt, BigUint>;

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;
    

    // Callbacks can only be declared here for the moment.

    #[callback]
    fn auction_stake_callback(&self, call_result: AsyncCallResult<()>) {
        self.contract_stake().auction_stake_callback(call_result);
    }

    #[callback]
    fn auction_unStake_callback(&self, call_result: AsyncCallResult<()>) {
        self.contract_stake().auction_unStake_callback(call_result);
    }

    #[callback]
    fn auction_unBond_callback(&self, call_result: AsyncCallResult<()>) {
        self.contract_stake().auction_unBond_callback(call_result);
    }
    
}
