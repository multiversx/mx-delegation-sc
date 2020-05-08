
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
    #[storage_set("nr_nodes")]
    fn _set_nr_nodes(&self, nr_nodes: usize);

    #[private]
    #[storage_set("bls_key")]
    fn _set_bls_key(&self, node_index: usize, bls_key: &Vec<u8>);

    #[private]
    #[storage_set("bls_sig")]
    fn _set_bls_signature(&self, node_index: usize, bls_signature: &Vec<u8>);

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
            nr_nodes: usize,
            #[multi(2*nr_nodes)] bls_keys_signatures: Vec<Vec<u8>>,
            #[payment] payment: &BigUint) -> Result<(), &str> {

        if self._is_staking_failure() {
            return Err("auction smart contract deliberate error");
        }

        self._set_received_stake(&payment);
        self._set_nr_nodes(nr_nodes);
        
        for n in 0..nr_nodes {
            self._set_bls_key(n, &bls_keys_signatures[2*n]);
            self._set_bls_signature(n, &bls_keys_signatures[2*n+1]);
        }

        Ok(())
    }
}
