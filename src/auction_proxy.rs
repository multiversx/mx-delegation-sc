
imports!();

use super::*;
use super::bls_key::*;

#[elrond_wasm_derive::callable(AuctionProxy)]
pub trait Auction {
    #[payable]
    #[callback(auction_stake_callback)]
    fn stake(&self,
        #[callback_arg] node_ids: Vec<usize>,
        num_nodes: usize,
        #[multi(2*num_nodes)] bls_keys_signatures: Vec<Vec<u8>>,
        #[payment] payment: &BigUint);

    #[callback(auction_unStake_callback)]
    fn unStake(&self,
        #[callback_arg] node_ids: Vec<usize>,
        #[var_args] bls_keys_signatures: Vec<BLSKey>);

    #[callback(auction_unBond_callback)]
    fn unBond(&self,
        #[callback_arg] node_ids: Vec<usize>,
        #[var_args] bls_keys_signatures: Vec<BLSKey>);
}
