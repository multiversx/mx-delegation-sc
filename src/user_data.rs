

// Groups together data per delegator from the storage.
pub struct UserData<BigUint> {
    /// The value of the total cumulated rewards in the contract when the user's rewards were computed the last time.
    pub reward_checkpoint: BigUint,

    /// Rewards that are computed but not yet sent to the delegator.
    pub unclaimed_rewards: BigUint,

    /// How much stake the delegator has in the contract.
    pub personal_stake: BigUint,
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
    fn _get_num_users(&self) -> usize;

    #[private]
    #[storage_set("num_users")]
    fn _set_num_users(&self, num_users: usize);

    // creates new user id
    #[private]
    fn new_user(&self) -> usize {
        let mut num_users = self._get_num_users();
        num_users += 1;
        self._set_num_users(num_users);
        num_users
    }

    /// Yields how many different addresses have staked in the contract.
    #[view]
    fn getNrDelegators(&self) -> usize {
        self._get_num_users() - 1
    }

    /// How much a delegator has staked.
    #[private]
    #[storage_get("u_stak")]
    fn _get_user_stake(&self, user_id: usize) -> BigUint;

    #[private]
    #[storage_set("u_stak")]
    fn _set_user_stake(&self, user_id: usize, user_stake: &BigUint);

    /// Claiming rewards has 2 steps:
    /// 1. computing the delegator rewards out of the total rewards, and
    /// 2. sending those rewards to the delegator address. 
    /// This field keeps track of rewards that went through step 1 but not 2,
    /// i.e. were computed and deducted from the total rewards, but not yet "physically" sent to the user.
    /// The unclaimed stake still resides in the contract.
    #[private]
    #[storage_get("u_uncl")]
    fn _get_user_unclaimed(&self, user_id: usize) -> BigUint;

    #[private]
    #[storage_set("u_uncl")]
    fn _set_user_unclaimed(&self, user_id: usize, user_unclaimed: &BigUint);

    /// As the time passes, if the contract is active, rewards periodically arrive in the contract. 
    /// Users can claim their share of rewards anytime.
    /// This field helps keeping track of how many rewards came to the contract since the last claim.
    /// More specifically, it indicates the cumulated sum of rewards that had arrived in the contract 
    /// when the user last claimed their own personal rewards.
    /// If zero, it means the user never claimed any rewards.
    /// If equal to getTotalCumulatedRewards, it means the user claimed everything there is for him/her.
    #[private]
    #[storage_get("u_last")]
    fn _get_user_last(&self, user_id: usize) -> BigUint;

    #[private]
    #[storage_set("u_last")]
    fn _set_user_last(&self, user_id: usize, user_last: &BigUint);

    /// Users can trade stake. To do so, a user must first offer stake for sale.
    /// This field keeps track of how much stake each user has offered for sale.
    #[private]
    #[storage_get("u_sale")]
    fn _get_user_stake_for_sale(&self, user_id: usize) -> BigUint;

    #[private]
    #[storage_set("u_sale")]
    fn _set_user_stake_for_sale(&self, user_id: usize, user_stake_for_sale: &BigUint);

    /// Loads the entire UserData object from storage.
    #[private]
    fn _load_user_data(&self, user_id: usize) -> UserData<BigUint> {
        let tot_rew = self._get_user_last(user_id);
        let per_rew = self._get_user_unclaimed(user_id);
        let per_stk = self._get_user_stake(user_id);
        UserData {
            reward_checkpoint: tot_rew,
            unclaimed_rewards: per_rew,
            personal_stake: per_stk,
        }
    }

    /// Saves a UserData object to storage.
    #[private]
    fn store_user_data(&self, user_id: usize, data: &UserData<BigUint>) {
        self._set_user_last(user_id, &data.reward_checkpoint);
        self._set_user_unclaimed(user_id, &data.unclaimed_rewards);
        self._set_user_stake(user_id, &data.personal_stake);
    }

    /// Block timestamp of when the user offered stake for sale.
    /// Note: not part of the UserData struct because it is not needed as often.
    #[private]
    #[storage_get("u_toff")]
    fn _get_user_time_of_stake_offer(&self, user_id: usize) -> u64;

    #[private]
    #[storage_set("u_toff")]
    fn _set_user_time_of_stake_offer(&self, user_id: usize, time_of_stake_offer: u64);

}