multiversx_sc::imports!();

use node_storage::types::{BLSKey, BLSSignature};

#[multiversx_sc::derive::proxy]
pub trait Auction {
    #[payable("EGLD")]
    #[endpoint]
    fn stake(
        &self,
        num_nodes: usize,
        bls_keys_signatures_args: &MultiValueEncoded<
            MultiValue2<BLSKey<Self::Api>, BLSSignature<Self::Api>>,
        >,
    ) -> MultiValueEncoded<ManagedBuffer>;

    #[endpoint(unStake)]
    fn unstake(
        &self,
        bls_keys: &MultiValueManagedVec<BLSKey<Self::Api>>,
    ) -> MultiValueEncoded<ManagedBuffer>;

    #[endpoint(unStakeNodes)]
    fn unstake_nodes(
        &self,
        bls_keys: &MultiValueManagedVec<BLSKey<Self::Api>>,
    ) -> MultiValueEncoded<ManagedBuffer>;

    #[endpoint(unBond)]
    fn unbond(
        &self,
        bls_keys: &MultiValueEncoded<BLSKey<Self::Api>>,
    ) -> MultiValueEncoded<ManagedBuffer>;

    #[endpoint(unBondNodes)]
    fn unbond_nodes(&self, bls_keys: &MultiValueManagedVec<BLSKey<Self::Api>>);

    #[endpoint(unStakeTokens)]
    fn unstake_tokens(&self, amount: &BigUint);

    #[endpoint(unBondTokens)]
    fn unbond_tokens(&self, amount: &BigUint);

    #[endpoint]
    fn claim(&self);

    #[payable("EGLD")]
    #[endpoint(unJail)]
    fn unjail(&self, bls_keys: MultiValueManagedVec<Self::Api, BLSKey<Self::Api>>);
}
