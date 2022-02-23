use node_storage::types::bls_key::BLSKey;

elrond_wasm::imports!();

#[elrond_wasm::derive::module]
pub trait AuctionMockStorage {
    #[storage_get("stake_per_node")]
    fn get_stake_per_node(&self) -> BigUint;

    #[storage_get("num_nodes")]
    fn get_num_nodes(&self) -> usize;

    #[storage_set("num_nodes")]
    fn set_num_nodes(&self, num_nodes: usize);

    #[storage_set("stake_bls_key")]
    fn set_stake_bls_key(&self, node_index: usize, bls_key: &[u8]);

    #[storage_set("stake_bls_sig")]
    fn set_stake_bls_signature(&self, node_index: usize, bls_signature: &[u8]);

    #[storage_set("unStake_bls_key")]
    fn set_unstake_bls_key(&self, node_index: usize, bls_key: &[u8]);

    #[storage_set("unBond_bls_key")]
    fn set_unbond_bls_key(&self, node_index: usize, bls_key: &[u8]);

    #[storage_set("staking_failure")]
    fn set_staking_failure(&self, will_fail: bool);

    #[storage_get("staking_failure")]
    fn is_staking_failure(&self) -> bool;

    #[endpoint(setBlsDeliberateError)]
    #[storage_set("bls_deliberate_error")]
    fn set_bls_deliberate_error(&self, bls_key: &[u8], err_code: u8);

    #[endpoint(getBlsDeliberateError)]
    #[storage_get("bls_deliberate_error")]
    fn get_bls_deliberate_error(&self, bls_key: &[u8]) -> u8;

    #[storage_set("unJailed")]
    fn set_unjailed(&self, bls_keys: &[BLSKey<Self::Api>]);
}
