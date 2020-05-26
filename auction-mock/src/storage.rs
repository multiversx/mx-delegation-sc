
#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![allow(unused_attributes)]

imports!();

#[elrond_wasm_derive::module(AuctionMockStorageImpl)]
pub trait AuctionMockStorage {

    #[private]
    #[storage_set("received_stake")]
    fn _set_received_stake(&self, total_stake: &BigUint);

    #[private]
    #[storage_get("num_nodes")]
    fn _get_num_nodes(&self) -> usize;

    #[private]
    #[storage_set("num_nodes")]
    fn _set_num_nodes(&self, num_nodes: usize);

    #[private]
    #[storage_set("stake_bls_key")]
    fn _set_stake_bls_key(&self, node_index: usize, bls_key: &Vec<u8>);

    #[private]
    #[storage_set("stake_bls_sig")]
    fn _set_stake_bls_signature(&self, node_index: usize, bls_signature: &Vec<u8>);

    #[private]
    #[storage_set("unStake_bls_key")]
    fn _set_unStake_bls_key(&self, node_index: usize, bls_key: &Vec<u8>);

    #[private]
    #[storage_set("unBond_bls_key")]
    fn _set_unBond_bls_key(&self, node_index: usize, bls_key: &Vec<u8>);

    #[private]
    #[storage_set("staking_failure")]
    fn setStakingFailure(&self, will_fail: bool);

    #[private]
    #[storage_get("staking_failure")]
    fn _is_staking_failure(&self) -> bool;

}
