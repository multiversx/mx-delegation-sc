
#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![allow(unused_attributes)]


#[elrond_wasm_derive::contract(AuctionMockImpl)]
pub trait AuctionMock {

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

    fn init(&self) {
    }

    #[payable]
    fn stake(&self,
            num_nodes: usize,
            #[multi(2*num_nodes)] bls_keys_signatures: Vec<Vec<u8>>,
            #[payment] payment: &BigUint) -> Result<(), &str> {

        if self._is_staking_failure() {
            return Err("auction smart contract deliberate error");
        }

        self._set_received_stake(&payment);
        self._set_num_nodes(num_nodes);
        
        for n in 0..num_nodes {
            self._set_stake_bls_key(n, &bls_keys_signatures[2*n]);
            self._set_stake_bls_signature(n, &bls_keys_signatures[2*n+1]);
        }

        Ok(())
    }

    fn unStake(&self,
            #[var_args] bls_keys: Vec<Vec<u8>>) -> Result<(), &str> {

        if self._is_staking_failure() {
            return Err("auction smart contract deliberate error");
        }

        let num_nodes = self._get_num_nodes();
        if num_nodes != bls_keys.len() {
            return Err("all BLS keys expected as arguments in this mock");
        }

        for n in 0..num_nodes {
            self._set_unStake_bls_key(n, &bls_keys[n]);
        }

        Ok(())
    }

    fn unBond(&self,
            #[var_args] bls_keys: Vec<Vec<u8>>) -> Result<(), &str> {

        if self._is_staking_failure() {
            return Err("auction smart contract deliberate error");
        }

        let num_nodes = self._get_num_nodes();
        if num_nodes != bls_keys.len() {
            return Err("all BLS keys expected as arguments in this mock");
        }

        for n in 0..num_nodes {
            self._set_unStake_bls_key(n, &bls_keys[n]);
        }

        Ok(())
    }
}
