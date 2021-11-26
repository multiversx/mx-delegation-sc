elrond_wasm::imports!();

use node_storage::types::{BLSKey, BLSSignature};

#[elrond_wasm::derive::proxy]
pub trait Auction {
    #[payable("EGLD")]
    #[endpoint]
    fn stake(
        &self,
        num_nodes: usize,
        #[var_args] bls_keys_signatures_args: ManagedVarArgs<
            MultiArg2<BLSKey<Self::Api>, BLSSignature<Self::Api>>,
        >,
    ) -> SCResult<MultiResultVec<BoxedBytes>>;

    #[endpoint(unStake)]
    fn unstake(
        &self,
        #[var_args] bls_keys: ManagedVarArgs<BLSKey<Self::Api>>,
    ) -> SCResult<MultiResultVec<BoxedBytes>>;

    #[endpoint(unStakeNodes)]
    fn unstake_nodes(
        &self,
        #[var_args] bls_keys: ManagedVarArgs<BLSKey<Self::Api>>,
    ) -> SCResult<MultiResultVec<BoxedBytes>>;

    #[endpoint(unBond)]
    fn unbond(
        &self,
        #[var_args] bls_keys: ManagedVarArgs<BLSKey<Self::Api>>,
    ) -> SCResult<MultiResultVec<BoxedBytes>>;

    #[endpoint(unBondNodes)]
    fn unbond_nodes(&self, #[var_args] bls_keys: ManagedVarArgs<BLSKey<Self::Api>>);

    #[endpoint(unStakeTokens)]
    fn unstake_tokens(&self, amount: &BigUint);

    #[endpoint(unBondTokens)]
    fn unbond_tokens(&self, amount: &BigUint);

    #[endpoint]
    fn claim(&self);

    #[payable("EGLD")]
    #[endpoint(unJail)]
    fn unjail(&self, #[var_args] bls_keys: ManagedVarArgsEager<Self::Api, BLSKey<Self::Api>>);
}
