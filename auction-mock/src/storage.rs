
#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![allow(unused_attributes)]

imports!();

#[elrond_wasm_derive::module(AuctionMockStorageImpl)]
pub trait AuctionMockStorage {

    #[storage_get("stake_per_node")]
    fn _get_stake_per_node(&self) -> BigUint;

    #[storage_get("num_nodes")]
    fn _get_num_nodes(&self) -> usize;

    #[storage_set("num_nodes")]
    fn _set_num_nodes(&self, num_nodes: usize);

    #[storage_set("stake_bls_key")]
    fn _set_stake_bls_key(&self, node_index: usize, bls_key: &Vec<u8>);

    #[storage_set("stake_bls_sig")]
    fn _set_stake_bls_signature(&self, node_index: usize, bls_signature: &Vec<u8>);

    #[storage_set("unStake_bls_key")]
    fn _set_unStake_bls_key(&self, node_index: usize, bls_key: &Vec<u8>);

    #[storage_set("unBond_bls_key")]
    fn _set_unBond_bls_key(&self, node_index: usize, bls_key: &Vec<u8>);

    #[storage_set("staking_failure")]
    fn setStakingFailure(&self, will_fail: bool);

    #[storage_get("staking_failure")]
    fn _is_staking_failure(&self) -> bool;

    #[endpoint]
    #[storage_set("bls_deliberate_error")]
    fn setBlsDeliberateError(&self, bls_key: &Vec<u8>, err_code: u8);

    #[endpoint]
    #[storage_get("bls_deliberate_error")]
    fn getBlsDeliberateError(&self, bls_key: &Vec<u8>) -> u8;

}
