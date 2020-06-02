
#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![allow(unused_attributes)]

mod storage;
use storage::*;

imports!();

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

        for (n, bls_key) in bls_keys.iter().enumerate() {
            self.storage()._set_unStake_bls_key(n, bls_key);
        }

        Ok(())
    }

    fn unBond(&self,
            #[var_args] bls_keys: Vec<Vec<u8>>) -> Result<(), &str> {

        if self.storage()._is_staking_failure() {
            return Err("auction smart contract deliberate error");
        }

        for (n, bls_key) in bls_keys.iter().enumerate() {
            self.storage()._set_unBond_bls_key(n, bls_key);
        }

        Ok(())
    }
}
