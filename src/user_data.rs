imports!();

use crate::user_stake_state::*;
use crate::unbond_queue::*;

// Groups together data per delegator from the storage.
pub struct UserData<BigUint> {
    /// The value of the total cumulated rewards in the contract when the user's rewards were computed the last time.
    pub reward_checkpoint: BigUint,

    /// Rewards that are computed but not yet sent to the delegator.
    pub unclaimed_rewards: BigUint,

    // /// How much stake the delegator has in the contract.
    pub active_stake: BigUint,
}

/// Storing total stake per type the same way as we store it for users, but with user_id 0.
/// There can be no user with id 0, so the value is safe to use.
/// These values are redundant. They help avoid having to recompute the sum, especially when computing rewards.
/// At all times the values stored here must be the sums of all user values for the respective stake state,
/// no operation may break this invariant!
pub const USER_STAKE_TOTALS_ID: usize = 0;

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
    #[private]
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

    /// How much of a delegator's has been sent to the auction SC.
    #[private]
    #[storage_get("u_stake_type")]
    fn _get_user_stake_of_type(&self, user_id: usize, stake_type: UserStakeState) -> BigUint;

    #[private]
    #[storage_set("u_stake_type")]
    fn _set_user_stake_of_type(&self, user_id: usize, stake_type: UserStakeState, stake: &BigUint);

    #[private]
    fn _increase_user_stake_of_type(&self, user_id: usize, stake_type: UserStakeState, amount: &BigUint) {
        let mut user_st_value = self._get_user_stake_of_type(user_id, stake_type);
        let mut total_st_value = self._get_user_stake_of_type(USER_STAKE_TOTALS_ID, stake_type);
        let mut user_total = self._get_user_total_stake(user_id);
        user_st_value += amount;
        total_st_value += amount;
        user_total += amount;
        self._set_user_stake_of_type(user_id, stake_type, &user_st_value);
        self._set_user_stake_of_type(USER_STAKE_TOTALS_ID, stake_type, &total_st_value);
        self._set_user_total_stake(user_id, &user_total);
    }

    #[private]
    fn _decrease_user_stake_of_type(&self, user_id: usize, stake_type: UserStakeState, amount: &BigUint) -> bool {
        let mut user_st_value = self._get_user_stake_of_type(user_id, stake_type);
        if amount > &user_st_value {
            return false;
        }
        let mut total_st_value = self._get_user_stake_of_type(USER_STAKE_TOTALS_ID, stake_type);
        let mut user_total = self._get_user_total_stake(user_id);
        user_st_value -= amount;
        total_st_value -= amount;
        user_total -= amount;
        self._set_user_stake_of_type(user_id, stake_type, &user_st_value);
        self._set_user_stake_of_type(USER_STAKE_TOTALS_ID, stake_type, &total_st_value);
        self._set_user_total_stake(user_id, &user_total);

        true
    }

    /// Yields how much a user has staked in the contract.
    #[view]
    fn getUserStake(&self, user_address: Address) -> BigUint {
        let user_id = self.getUserId(&user_address);
        if user_id == 0 {
            BigUint::zero()
        } else {
            self._get_user_total_stake(user_id)
        }
    }

    #[view]
    fn getUserActiveStake(&self, user_address: Address) -> BigUint {
        let user_id = self.getUserId(&user_address);
        if user_id == 0 {
            BigUint::zero()
        } else {
            self._get_user_stake_of_type(user_id, UserStakeState::Active)
        }
    }

    #[view]
    fn getUserInactiveStake(&self, user_address: Address) -> BigUint {
        let user_id = self.getUserId(&user_address);
        if user_id == 0 {
            BigUint::zero()
        } else {
            self._get_user_stake_of_type(user_id, UserStakeState::Inactive) +
            self._get_user_stake_of_type(user_id, UserStakeState::WithdrawOnly)
        }
    }

    #[private]
    fn _get_user_stake_by_type(&self, user_id: usize) -> Vec<BigUint> {
        let mut result = Vec::<BigUint>::with_capacity(7);
        result.push(self._get_user_stake_of_type(user_id, UserStakeState::Inactive));
        result.push(self._get_user_stake_of_type(user_id, UserStakeState::PendingActivation));
        result.push(self._get_user_stake_of_type(user_id, UserStakeState::Active));
        result.push(self._get_user_stake_of_type(user_id, UserStakeState::PendingDeactivation));
        result.push(self._get_user_stake_of_type(user_id, UserStakeState::UnBondPeriod));
        result.push(self._get_user_stake_of_type(user_id, UserStakeState::PendingUnBond));
        result.push(self._get_user_stake_of_type(user_id, UserStakeState::WithdrawOnly));
        result.push(self._get_user_stake_of_type(user_id, UserStakeState::ActivationFailed));
        result
    }

    #[view]
    fn getUserStakeByType(&self, user_address: &Address) -> MultiResultVec<BigUint> {
        // TODO: replace result type with something based on tuples
        let user_id = self.getUserId(&user_address);
        if user_id == 0 {
            let mut result = Vec::<BigUint>::with_capacity(7);
            result.push(BigUint::zero());
            result.push(BigUint::zero());
            result.push(BigUint::zero());
            result.push(BigUint::zero());
            result.push(BigUint::zero());
            result.push(BigUint::zero());
            result.push(BigUint::zero());
            result.push(BigUint::zero());
            result
        } else {
            self._get_user_stake_by_type(user_id)
        }
    }

    #[view]
    fn getTotalStakeByType(&self) -> MultiResultVec<BigUint> {
        self._get_user_stake_by_type(USER_STAKE_TOTALS_ID)
    }

    #[view]
    fn getTotalActiveStake(&self) -> BigUint {
        self._get_user_stake_of_type(USER_STAKE_TOTALS_ID, UserStakeState::Active)
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
        let u_stake_actv = self._get_user_stake_of_type(user_id, UserStakeState::Active);
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
        self._set_user_stake_of_type(user_id, UserStakeState::Active, &data.active_stake);
    }

    /// Block timestamp of when the user offered stake for sale.
    /// Note: not part of the UserData struct because it is not needed as often.
    #[private]
    #[storage_get("u_stake_toff")]
    fn _get_user_bl_nonce_of_stake_offer(&self, user_id: usize) -> u64;

    #[private]
    #[storage_set("u_stake_toff")]
    fn _set_user_bl_nonce_of_stake_offer(&self, user_id: usize, bl_nonce_of_stake_offer: u64);

    #[private]
    fn convert_user_stake(&self, user_id: usize, old_type: UserStakeState, new_type: UserStakeState, total_supply: &mut BigUint) {
        let mut user_stake_old_type = self._get_user_stake_of_type(user_id, old_type);
        let mut user_stake_new_type = self._get_user_stake_of_type(user_id, new_type);
        let mut total_stake_old_type = self._get_user_stake_of_type(USER_STAKE_TOTALS_ID, old_type);
        let mut total_stake_new_type = self._get_user_stake_of_type(USER_STAKE_TOTALS_ID, new_type);
        if &*total_supply > &user_stake_old_type {
            user_stake_new_type += &user_stake_old_type;
            total_stake_new_type += &user_stake_old_type;
            total_stake_old_type -= &user_stake_old_type;
            *total_supply -= &user_stake_old_type;
            user_stake_old_type = BigUint::zero();
        } else {
            user_stake_old_type -= &*total_supply;
            total_stake_old_type -= &*total_supply;
            user_stake_new_type += &*total_supply;
            total_stake_new_type += &*total_supply;
            *total_supply = BigUint::zero();
        }
        self._set_user_stake_of_type(user_id, old_type, &user_stake_old_type);
        self._set_user_stake_of_type(user_id, new_type, &user_stake_new_type);
        self._set_user_stake_of_type(USER_STAKE_TOTALS_ID, old_type, &total_stake_old_type);
        self._set_user_stake_of_type(USER_STAKE_TOTALS_ID, new_type, &total_stake_new_type);
    }

    /// Converts from one type of stake to another, for as many users as possible,
    /// within the limit of total_supply.
    /// Argument total_supply decreases in the process.
    /// Walking in increasing user id order, so older users get picked first.
    #[private]
    fn convert_user_stake_asc(&self, old_type: UserStakeState, new_type: UserStakeState, total_supply: &mut BigUint) {
        let num_users = self.getNumUsers();
        let mut i = 1usize;
        while i <= num_users && *total_supply > 0 {
            self.convert_user_stake(i, old_type, new_type, total_supply);
            i += 1;
        }
    }

    /// Converts from one type of stake to another, for as many users as possible,
    /// within the limit of total_supply.
    /// Argument total_supply decreases in the process.
    /// Walking in decreasing user id order, so newer users get picked first.
    #[private]
    fn convert_user_stake_desc(&self, old_type: UserStakeState, new_type: UserStakeState, total_supply: &mut BigUint) {
        let mut i = self.getNumUsers();
        while i > 0 && *total_supply > 0 {
            self.convert_user_stake(i, old_type, new_type, total_supply);
            i -= 1;
        }
    }

    #[view]
    #[storage_get("unbond_queue")]
    fn getUnbondQueue(&self) -> Vec<UnbondQueueItem<BigUint>>;

    #[private]
    #[storage_set("unbond_queue")]
    fn _set_unbond_queue(&self, unbond_queue: &[UnbondQueueItem<BigUint>]);

    
}