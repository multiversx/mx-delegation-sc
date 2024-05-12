#![no_std]
#![allow(clippy::type_complexity)]

mod storage;

use node_storage::types::bls_key::BLSKey;

multiversx_sc::imports!();

#[multiversx_sc::derive::contract]
pub trait AuctionMock: storage::AuctionMockStorage {
    #[init]
    fn init(&self) {}

    #[payable("EGLD")]
    #[endpoint]
    fn stake(
        &self,
        num_nodes: usize,
        bls_keys_signatures: MultiValueEncoded<MultiValue2<ManagedBuffer, ManagedBuffer>>,
        #[payment] payment: BigUint,
    ) -> MultiValueEncoded<ManagedBuffer> {
        require!(
            num_nodes * 2 == bls_keys_signatures.raw_len(),
            "incorrect number of arguments"
        );

        require!(
            !self.is_staking_failure(),
            "auction smart contract deliberate error"
        );

        let mut new_num_nodes = self.get_num_nodes();
        let expected_payment = BigUint::from(num_nodes as u64) * self.get_stake_per_node();
        require!(
            payment == expected_payment,
            "incorrect payment to auction mock"
        );

        let mut result_err_data: MultiValueEncoded<ManagedBuffer> = MultiValueEncoded::new();
        for key_sig_pair in bls_keys_signatures.into_iter() {
            new_num_nodes += 1;
            let (bls_key, bls_sig) = key_sig_pair.into_tuple();
            self.set_stake_bls_key(new_num_nodes, bls_key.to_boxed_bytes().as_slice());
            self.set_stake_bls_signature(new_num_nodes, bls_sig.to_boxed_bytes().as_slice());

            let err_code = self.get_bls_deliberate_error(bls_key.to_boxed_bytes().as_slice());
            if err_code > 0 {
                result_err_data.push(bls_key);
                result_err_data.push(ManagedBuffer::from(&[err_code][..]));
            }
        }

        self.set_num_nodes(new_num_nodes);

        result_err_data
    }

    #[endpoint(unStake)]
    fn unstake_endpoint(
        &self,
        bls_keys: MultiValueEncoded<ManagedBuffer>,
    ) -> MultiValueEncoded<ManagedBuffer> {
        require!(
            !self.is_staking_failure(),
            "auction smart contract deliberate error"
        );

        let mut result_err_data: MultiValueEncoded<ManagedBuffer> = MultiValueEncoded::new();
        for (n, bls_key) in bls_keys.into_iter().enumerate() {
            self.set_unstake_bls_key(n, bls_key.to_boxed_bytes().as_slice());

            let err_code = self.get_bls_deliberate_error(bls_key.to_boxed_bytes().as_slice());
            if err_code > 0 {
                result_err_data.push(bls_key.clone());
                result_err_data.push(ManagedBuffer::from(&[err_code][..]));
            }
        }

        result_err_data
    }

    #[endpoint(unStakeNodes)]
    fn unstake_nodes_endpoint(
        &self,
        bls_keys: MultiValueEncoded<ManagedBuffer>,
    ) -> MultiValueEncoded<ManagedBuffer> {
        self.unstake_endpoint(bls_keys)
    }

    #[endpoint(unBond)]
    fn unbond_endpoint(
        &self,
        bls_keys: MultiValueEncoded<ManagedBuffer>,
    ) -> MultiValueEncoded<ManagedBuffer> {
        require!(
            !self.is_staking_failure(),
            "auction smart contract deliberate error"
        );

        let mut result_err_data: MultiValueEncoded<ManagedBuffer> = MultiValueEncoded::new();
        let bls_keys_len = bls_keys.len();
        for (n, bls_key) in bls_keys.into_iter().enumerate() {
            self.set_unbond_bls_key(n, bls_key.to_boxed_bytes().as_slice());

            let err_code = self.get_bls_deliberate_error(bls_key.to_boxed_bytes().as_slice());
            if err_code > 0 {
                result_err_data.push(bls_key.clone());
                result_err_data.push(ManagedBuffer::from(&[err_code][..]));
            }
        }

        let unbond_stake = self.get_stake_per_node() * BigUint::from(bls_keys_len);
        self.tx().to(ToCaller).egld(unbond_stake).transfer();

        result_err_data
    }

    #[endpoint(unBondNodes)]
    fn unbond_nodes_endpoint(
        &self,
        bls_keys: MultiValueEncoded<ManagedBuffer>,
    ) -> MultiValueEncoded<ManagedBuffer> {
        self.unbond_endpoint(bls_keys)
    }

    #[endpoint(unStakeTokens)]
    fn unstake_tokens(&self, _amount: BigUint) {}

    #[endpoint(unBondTokens)]
    fn unbond_tokens(&self, amount: BigUint) {
        self.tx().to(ToCaller).egld(amount).transfer();
    }

    #[endpoint]
    fn claim(&self) {}

    #[payable("EGLD")]
    #[endpoint(unJail)]
    fn unjail_endpoint(
        &self,
        bls_keys: MultiValueManagedVec<BLSKey<Self::Api>>,
        #[payment] _fine_payment: BigUint,
    ) {
        self.set_unjailed(&bls_keys.into_vec());
    }
}
