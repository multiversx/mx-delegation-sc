
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

        let mut new_num_nodes = self.storage()._get_num_nodes();
        let expected_payment = BigUint::from(num_nodes) * self.storage()._get_stake_per_node();
        if payment != &expected_payment {
            return Err("incorrect payment to auction mock");
        }

        for n in 0..num_nodes {
            new_num_nodes += 1;
            self.storage()._set_stake_bls_key(new_num_nodes, &bls_keys_signatures[2*n]);
            self.storage()._set_stake_bls_signature(new_num_nodes, &bls_keys_signatures[2*n+1]);
        }

        self.storage()._set_num_nodes(new_num_nodes);

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

        let unbond_stake = BigUint::from(bls_keys.len()) * self.storage()._get_stake_per_node();
        self.send_tx(&self.get_caller(), &unbond_stake, "unbond stake");

        Ok(())
    }
}
