
#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![allow(unused_attributes)]

mod storage;
use storage::*;

#[elrond_wasm_derive::contract(AuctionMockImpl)]
pub trait AuctionMock {

    #[module(AuctionMockStorageImpl)]
    fn storage(&self) -> AuctionMockStorageImpl<T, BigInt, BigUint>;

    fn init(&self) {
    }

    #[payable]
    fn stake(&self,
            num_nodes: usize,
            #[multi(2*num_nodes)] bls_keys_signatures: Vec<Vec<u8>>,
            #[payment] payment: &BigUint) -> Result<(), &str> {

        if self.storage()._is_staking_failure() {
            return Err("auction smart contract deliberate error");
        }

        self.storage()._set_received_stake(&payment);
        self.storage()._set_num_nodes(num_nodes);
        
        for n in 0..num_nodes {
            self.storage()._set_stake_bls_key(n, &bls_keys_signatures[2*n]);
            self.storage()._set_stake_bls_signature(n, &bls_keys_signatures[2*n+1]);
        }

        Ok(())
    }

    fn unStake(&self,
            #[var_args] bls_keys: Vec<Vec<u8>>) -> Result<(), &str> {

        if self.storage()._is_staking_failure() {
            return Err("auction smart contract deliberate error");
        }

        let num_nodes = self.storage()._get_num_nodes();
        if num_nodes != bls_keys.len() {
            return Err("all BLS keys expected as arguments in this mock");
        }

        for n in 0..num_nodes {
            self.storage()._set_unStake_bls_key(n, &bls_keys[n]);
        }

        Ok(())
    }

    fn unBond(&self,
            #[var_args] bls_keys: Vec<Vec<u8>>) -> Result<(), &str> {

        if self.storage()._is_staking_failure() {
            return Err("auction smart contract deliberate error");
        }

        let num_nodes = self.storage()._get_num_nodes();
        if num_nodes != bls_keys.len() {
            return Err("all BLS keys expected as arguments in this mock");
        }

        for n in 0..num_nodes {
            self.storage()._set_unStake_bls_key(n, &bls_keys[n]);
        }

        Ok(())
    }
}
