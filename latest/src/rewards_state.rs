use crate::settings::{OWNER_USER_ID, PERCENTAGE_DENOMINATOR};

use core::num::NonZeroUsize;
use user_fund_storage::fund_view_module::USER_STAKE_TOTALS_ID;
use user_fund_storage::types::FundType;

multiversx_sc::imports!();

// Groups together data per delegator from the storage.
pub struct UserRewardData<BigUint> {
    /// The value of the total cumulated rewards in the contract when the user's rewards were computed the last time.
    pub reward_checkpoint: BigUint,

    /// Rewards that are computed but not yet sent to the delegator.
    pub unclaimed_rewards: BigUint,
}

/// Contains logic to compute and distribute individual delegator rewards.
#[multiversx_sc::derive::module]
pub trait RewardStateModule:
    crate::settings::SettingsModule
    + user_fund_storage::user_data::UserDataModule
    + user_fund_storage::fund_module::FundModule
    + user_fund_storage::fund_view_module::FundViewModule
{
    /// Claiming rewards has 2 steps:
    /// 1. computing the delegator rewards out of the total rewards, and
    /// 2. sending those rewards to the delegator address.
    /// This field keeps track of rewards that went through step 1 but not 2,
    /// i.e. were computed and deducted from the total rewards, but not yet "physically" sent to the user.
    /// The unclaimed stake still resides in the contract.
    #[storage_get("u_rew_unclmd")]
    fn get_user_rew_unclaimed(&self, user_id: NonZeroUsize) -> BigUint;

    #[storage_set("u_rew_unclmd")]
    fn set_user_rew_unclaimed(&self, user_id: NonZeroUsize, user_rew_unclaimed: &BigUint);

    /// As the time passes, if the contract is active, rewards periodically arrive in the contract.
    /// Users can claim their share of rewards anytime.
    /// This field helps keeping track of how many rewards came to the contract since the last claim.
    /// More specifically, it indicates the cumulated sum of rewards that had arrived in the contract
    /// when the user last claimed their own personal rewards.
    /// If zero, it means the user never claimed any rewards.
    /// If equal to get_total_cumulated_rewards, it means the user claimed everything there is for him/her.
    #[storage_get("u_rew_checkp")]
    fn get_user_rew_checkpoint(&self, user_id: NonZeroUsize) -> BigUint;

    #[storage_set("u_rew_checkp")]
    fn set_user_rew_checkpoint(&self, user_id: NonZeroUsize, user_rew_checkpoint: &BigUint);

    #[storage_get("sent_rewards")]
    fn get_sent_rewards(&self) -> BigUint;

    #[storage_set("sent_rewards")]
    fn set_sent_rewards(&self, sent_rewards: &BigUint);

    /// Yields all the rewards received by the contract since its creation.
    /// This value is monotonously increasing - it can never decrease.
    /// Handing out rewards will not decrease this value.
    /// This is to keep track of how many funds entered the contract. It ignores any funds leaving the contract.
    /// Individual rewards are computed based on this value.
    /// For each user we keep a record on what was the value of the historical rewards when they last claimed.
    /// Subtracting that from the current historical rewards yields how much accumulated in the contract since they last claimed.
    #[view(getTotalCumulatedRewards)]
    fn get_total_cumulated_rewards(&self) -> BigUint {
        self.blockchain().get_cumulated_validator_rewards()
    }

    /// The account running the nodes is entitled to (service_fee / NODE_DENOMINATOR) * rewards.
    /// Yields the service reward and the non-service-reward.
    ///
    /// The sum of the 2 outputs is <= tot_rewards (not always equal).
    /// Both results are rounded down,
    /// so te rounding error is not in the result.
    /// This is deliberate, to avoid a very subtle rounding error edge case.
    fn split_service_reward(&self, tot_rewards: &BigUint) -> (BigUint, BigUint) {
        let service_fee = &self.get_service_fee();
        let perc_denominator = &BigUint::from(PERCENTAGE_DENOMINATOR);

        // part of the rewards that goes to the owner
        let mut service_rewards = service_fee * tot_rewards;
        service_rewards /= perc_denominator;

        // part of the rewards that gets split amongst delegators
        let mut total_delegators_rewards = perc_denominator - service_fee;
        total_delegators_rewards *= tot_rewards;
        total_delegators_rewards /= perc_denominator;

        (service_rewards, total_delegators_rewards)
    }

    /// Does not update storage, only returns the user rewards object, after computing rewards.
    fn load_updated_user_rewards(&self, user_id: NonZeroUsize) -> UserRewardData<BigUint> {
        let mut user_data = self.load_user_reward_data(user_id);

        // new rewards are what was added since the last time rewards were computed
        let tot_cumul_rewards = self.get_total_cumulated_rewards();
        let tot_new_rewards = &tot_cumul_rewards - &user_data.reward_checkpoint;
        if tot_new_rewards == 0 {
            return user_data; // nothing happened since the last claim
        }

        // the owner is entitled to: tot_new_rewards * service_fee / NODE_DENOMINATOR
        // delegators are entitled to: tot_new_rewards * (1 - service_fee / NODE_DENOMINATOR)
        let (service_rewards, total_delegators_rewards) =
            self.split_service_reward(&tot_new_rewards);

        let delegation_cap = self.get_total_delegation_cap();

        // update node rewards, if applicable
        if user_id == OWNER_USER_ID {
            // the owner gets the service fee
            user_data.unclaimed_rewards += &service_rewards;

            // the owner gets the rewards for the missing active (unstaked) stake
            let tot_stake_active =
                self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active);
            if tot_stake_active < delegation_cap {
                let missing_stake = &delegation_cap - &tot_stake_active;
                let mut owner_unstaked_rewards = total_delegators_rewards.clone();
                owner_unstaked_rewards *= &missing_stake;
                owner_unstaked_rewards /= &delegation_cap;
                user_data.unclaimed_rewards += &owner_unstaked_rewards;
            }
        }

        // update delegator rewards based on Active stake
        let u_stake_active = self.get_user_stake_of_type(user_id.get(), FundType::Active);
        if u_stake_active > 0 {
            // delegator reward is:
            // total new rewards * (1 - service_fee / NODE_DENOMINATOR) * user stake / total delegation cap
            let mut delegator_new_rewards = total_delegators_rewards;
            delegator_new_rewards *= &u_stake_active;
            delegator_new_rewards /= &delegation_cap;
            user_data.unclaimed_rewards += &delegator_new_rewards;
        }

        // update user data checkpoint
        user_data.reward_checkpoint = tot_cumul_rewards;

        user_data
    }

    /// Convenience method, brings user rewards up to date for one user.
    fn compute_one_user_reward(&self, user_id: NonZeroUsize) {
        let user_data = self.load_updated_user_rewards(user_id);
        self.store_user_reward_data(user_id, &user_data);
    }

    /// Yields how much a user is able to claim in rewards at the present time.
    /// Does not update storage.
    #[view(getClaimableRewards)]
    fn get_claimable_rewards(&self, user: ManagedAddress) -> BigUint {
        if let Some(user_id) = NonZeroUsize::new(self.get_user_id(&user)) {
            let user_data = self.load_updated_user_rewards(user_id);
            user_data.unclaimed_rewards
        } else {
            BigUint::zero()
        }
    }

    /// Utility readonly function to check how many unclaimed rewards currently reside in the contract.
    #[view(getTotalUnclaimedRewards)]
    fn get_total_unclaimed_rewards(&self) -> BigUint {
        let num_users = self.get_num_users();
        let mut sum_unclaimed = BigUint::zero();

        // regular rewards
        for user_id in NonZeroUsizeIterator::from_1_to_n(num_users) {
            let user_data = self.load_updated_user_rewards(user_id);
            sum_unclaimed += user_data.unclaimed_rewards;
        }

        sum_unclaimed
    }

    /// Loads the entire UserRewardData object from storage.
    fn load_user_reward_data(&self, user_id: NonZeroUsize) -> UserRewardData<BigUint> {
        let u_rew_checkp = self.get_user_rew_checkpoint(user_id);
        let u_rew_unclmd = self.get_user_rew_unclaimed(user_id);
        UserRewardData {
            reward_checkpoint: u_rew_checkp,
            unclaimed_rewards: u_rew_unclmd,
        }
    }

    /// Saves a UserRewardData object to storage.
    fn store_user_reward_data(&self, user_id: NonZeroUsize, data: &UserRewardData<BigUint>) {
        self.set_user_rew_checkpoint(user_id, &data.reward_checkpoint);
        self.set_user_rew_unclaimed(user_id, &data.unclaimed_rewards);
    }

    #[view(getTotalUnProtected)]
    fn total_unprotected(&self) -> BigUint {
        let sent_rewards = self.get_sent_rewards();
        let total_rewards = self.get_total_cumulated_rewards();
        let total_waiting = self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Waiting);
        let total_deferred =
            self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::DeferredPayment);
        let total_withdraw =
            self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::WithdrawOnly);

        let mut unprotected = self
            .blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(), 0)
            + sent_rewards;
        unprotected -= total_rewards;
        unprotected -= total_waiting;
        unprotected -= total_deferred;
        unprotected -= total_withdraw;
        unprotected
    }
}
