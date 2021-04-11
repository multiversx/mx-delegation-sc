elrond_wasm::imports!();

use super::node_storage::types::*;

#[elrond_wasm_derive::callable(AuctionProxy)]
pub trait Auction {
    #[payable("EGLD")]
    fn stake(
        &self,
        num_nodes: usize,
        #[var_args] bls_keys_signatures: VarArgs<MultiArg2<BLSKey, BLSSignature>>,
    ) -> ContractCall<BigUint, ()>;

    fn unStake(&self, #[var_args] bls_keys: VarArgs<BLSKey>) -> ContractCall<BigUint, ()>;

    fn unStakeNodes(&self, #[var_args] bls_keys: VarArgs<BLSKey>) -> ContractCall<BigUint, ()>;

    fn unBond(&self, #[var_args] bls_keys: VarArgs<BLSKey>) -> ContractCall<BigUint, ()>;

    fn claim(&self) -> ContractCall<BigUint, ()>;

    #[payable("EGLD")]
    fn unJail(&self, #[var_args] bls_keys: VarArgs<BLSKey>) -> ContractCall<BigUint, ()>;
}
