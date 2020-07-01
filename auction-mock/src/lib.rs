
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

    #[init]
    fn init(&self) {
    }

    #[payable]
    #[endpoint]
    fn stake(&self,
            num_nodes: usize,
            #[multi(2*num_nodes)] bls_keys_signatures_args: VarArgs<Vec<u8>>,
            #[payment] payment: &BigUint) -> Result<MultiResultVec<Vec<u8>>, SCError> {

        let bls_keys_signatures = bls_keys_signatures_args.into_vec();

        if self.storage()._is_staking_failure() {
            return sc_error!("auction smart contract deliberate error");
        }

        let mut new_num_nodes = self.storage()._get_num_nodes();
        let expected_payment = BigUint::from(num_nodes) * self.storage()._get_stake_per_node();
        if payment != &expected_payment {
            return sc_error!("incorrect payment to auction mock");
        }

        let mut result_err_data: Vec<Vec<u8>> = Vec::new();
        for n in 0..num_nodes {
            new_num_nodes += 1;
            let bls_key = &bls_keys_signatures[2*n];
            self.storage()._set_stake_bls_key(new_num_nodes, bls_key);
            let bls_sig = &bls_keys_signatures[2*n+1];
            self.storage()._set_stake_bls_signature(new_num_nodes, bls_sig);

            let err_code = self.storage().getBlsDeliberateError(bls_key);
            if err_code > 0 {
                result_err_data.push(bls_key.clone());
                result_err_data.push([err_code].to_vec());
            }
        }

        self.storage()._set_num_nodes(new_num_nodes);

        Ok(result_err_data.into())
    }

    #[endpoint]
    fn unStake(&self,
            #[var_args] bls_keys: VarArgs<Vec<u8>>) -> Result<MultiResultVec<Vec<u8>>, SCError> {

        if self.storage()._is_staking_failure() {
            return sc_error!("auction smart contract deliberate error");
        }

        let mut result_err_data: Vec<Vec<u8>> = Vec::new();
        for (n, bls_key) in bls_keys.iter().enumerate() {
            self.storage()._set_unStake_bls_key(n, bls_key);

            let err_code = self.storage().getBlsDeliberateError(bls_key);
            if err_code > 0 {
                result_err_data.push(bls_key.clone());
                result_err_data.push([err_code].to_vec());
            }
        }

        Ok(result_err_data.into())
    }

    #[endpoint]
    fn unBond(&self,
            #[var_args] bls_keys: VarArgs<Vec<u8>>) -> Result<MultiResultVec<Vec<u8>>, SCError> {

        if self.storage()._is_staking_failure() {
            return sc_error!("auction smart contract deliberate error");
        }

        let mut result_err_data: Vec<Vec<u8>> = Vec::new();
        for (n, bls_key) in bls_keys.iter().enumerate() {
            self.storage()._set_unBond_bls_key(n, bls_key);

            let err_code = self.storage().getBlsDeliberateError(bls_key);
            if err_code > 0 {
                result_err_data.push(bls_key.clone());
                result_err_data.push([err_code].to_vec());
            }
        }

        let unbond_stake = BigUint::from(bls_keys.len()) * self.storage()._get_stake_per_node();
        self.send_tx(&self.get_caller(), &unbond_stake, "unbond stake");

        Ok(result_err_data.into())
    }

    #[endpoint]
    fn claim(&self) -> Result<(), SCError> {
        Ok(())
    }
}
