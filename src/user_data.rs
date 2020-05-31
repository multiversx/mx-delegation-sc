imports!();

use crate::node_state::*;

// Groups together data per delegator from the storage.
pub struct UserData<BigUint> {
    /// The value of the total cumulated rewards in the contract when the user's rewards were computed the last time.
    pub reward_checkpoint: BigUint,

    /// Rewards that are computed but not yet sent to the delegator.
    pub unclaimed_rewards: BigUint,

    // /// How much stake the delegator has in the contract.
    pub active_stake: BigUint,
}

/// Deals with storage data about delegators.
#[elrond_wasm_derive::module(UserDataModuleImpl)]
pub trait UserDataModule {

    /// Each delegator gets a user id. This is in order to be able to iterate over their data.
    /// This is a mapping from delegator address to delegator id.
    /// The key is the bytes "user_id" concatenated with their public key.
    /// The value is the user id.
    #[storage_get("user_id")]
    fn getUserId(&self, address: &Address) -> usize;

    #[private]
    #[storage_set("user_id")]
    fn _set_user_id(&self, address: &Address, user_id: usize);

    /// Nr delegators + 1 (the node address)
    #[private]
    #[storage_get("num_users")]
    fn getNumUsers(&self) -> usize;

    /// Yields how accounts are registered in the contract.
    /// Note that not all of them must have stakes greater than zero.
    #[view]
    #[storage_set("num_users")]
    fn _set_num_users(&self, num_users: usize);

    // creates new user id
    #[private]
    fn new_user(&self) -> usize {
        let mut num_users = self.getNumUsers();
        num_users += 1;
        self._set_num_users(num_users);
        num_users
    }

    /// How much a delegator has staked.
    #[private]
    #[storage_get("u_stake_totl")]
    fn _get_user_total_stake(&self, user_id: usize) -> BigUint;

    #[private]
    #[storage_set("u_stake_totl")]
    fn _set_user_total_stake(&self, user_id: usize, user_total_stake: &BigUint);

    // TODO: auto-generate
    #[private]
    fn _update_user_total_stake<F: FnOnce(&mut BigUint)>(&self, user_id: usize, f: F)
    {
        let mut value = self._get_user_total_stake(user_id);
        f(&mut value);
        self._set_user_total_stake(user_id, &value);
    }

    /// How much of a delegator's has been sent to the auction SC.
    #[private]
    #[storage_get("u_stake_type")]
    fn _get_user_stake_of_type(&self, user_id: usize, stake_type: NodeState) -> BigUint;

    #[private]
    #[storage_set("u_stake_type")]
    fn _set_user_stake_of_type(&self, user_id: usize, stake_type: NodeState, stake: &BigUint);

    // TODO: auto-generate
    #[private]
    fn _update_user_active_stake<F, R>(&self, user_id: usize, f: F) -> R
    where F: Fn(&mut BigUint) -> R
    {
        let mut value = self._get_user_stake_of_type(user_id, NodeState::Active);
        let result = f(&mut value);
        self._set_user_stake_of_type(user_id, NodeState::Active, &value);
        result
    }

    /// Claiming rewards has 2 steps:
    /// 1. computing the delegator rewards out of the total rewards, and
    /// 2. sending those rewards to the delegator address. 
    /// This field keeps track of rewards that went through step 1 but not 2,
    /// i.e. were computed and deducted from the total rewards, but not yet "physically" sent to the user.
    /// The unclaimed stake still resides in the contract.
    #[private]
    #[storage_get("u_rew_unclmd")]
    fn _get_user_rew_unclaimed(&self, user_id: usize) -> BigUint;

    #[private]
    #[storage_set("u_rew_unclmd")]
    fn _set_user_rew_unclaimed(&self, user_id: usize, user_rew_unclaimed: &BigUint);

    /// As the time passes, if the contract is active, rewards periodically arrive in the contract. 
    /// Users can claim their share of rewards anytime.
    /// This field helps keeping track of how many rewards came to the contract since the last claim.
    /// More specifically, it indicates the cumulated sum of rewards that had arrived in the contract 
    /// when the user last claimed their own personal rewards.
    /// If zero, it means the user never claimed any rewards.
    /// If equal to getTotalCumulatedRewards, it means the user claimed everything there is for him/her.
    #[private]
    #[storage_get("u_rew_checkp")]
    fn _get_user_rew_checkpoint(&self, user_id: usize) -> BigUint;

    #[private]
    #[storage_set("u_rew_checkp")]
    fn _set_user_rew_checkpoint(&self, user_id: usize, user_rew_checkpoint: &BigUint);

    /// Users can trade stake. To do so, a user must first offer stake for sale.
    /// This field keeps track of how much stake each user has offered for sale.
    #[private]
    #[storage_get("u_stake_sale")]
    fn _get_user_stake_for_sale(&self, user_id: usize) -> BigUint;

    #[private]
    #[storage_set("u_stake_sale")]
    fn _set_user_stake_for_sale(&self, user_id: usize, user_stake_for_sale: &BigUint);

    // TODO: auto-generate
    #[private]
    fn _update_user_stake_for_sale<F, R>(&self, user_id: usize, f: F) -> R
    where F: Fn(&mut BigUint) -> R
    {
        let mut value = self._get_user_stake_for_sale(user_id);
        let result = f(&mut value);
        self._set_user_stake_for_sale(user_id, &value);
        result
    }

    /// Loads the entire UserData object from storage.
    #[private]
    fn _load_user_data(&self, user_id: usize) -> UserData<BigUint> {
        let u_rew_checkp = self._get_user_rew_checkpoint(user_id);
        let u_rew_unclmd = self._get_user_rew_unclaimed(user_id);
        // let u_stake_totl = self._get_user_total_stake(user_id);
        let u_stake_actv = self._get_user_stake_of_type(user_id, NodeState::Active);
        UserData {
            reward_checkpoint: u_rew_checkp,
            unclaimed_rewards: u_rew_unclmd,
            // total_stake: u_stake_totl,
            active_stake: u_stake_actv,
        }
    }

    /// Saves a UserData object to storage.
    #[private]
    fn store_user_data(&self, user_id: usize, data: &UserData<BigUint>) {
        self._set_user_rew_checkpoint(user_id, &data.reward_checkpoint);
        self._set_user_rew_unclaimed(user_id, &data.unclaimed_rewards);
        // self._set_user_total_stake(user_id, &data.total_stake);
        self._set_user_stake_of_type(user_id, NodeState::Active, &data.active_stake);
    }

    /// Block timestamp of when the user offered stake for sale.
    /// Note: not part of the UserData struct because it is not needed as often.
    #[private]
    #[storage_get("u_stake_toff")]
    fn _get_user_time_of_stake_offer(&self, user_id: usize) -> u64;

    #[private]
    #[storage_set("u_stake_toff")]
    fn _set_user_time_of_stake_offer(&self, user_id: usize, time_of_stake_offer: u64);

}