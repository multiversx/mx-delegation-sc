
use crate::user_stake_state::*;

use super::settings::*;
use crate::events::*;
use crate::node_config::*;
use crate::user_stake::*;
use crate::node_activation::*;
use crate::user_data::*;

imports!();

/// Contains logic to compute and distribute individual delegator rewards.
#[elrond_wasm_derive::module(RewardsModuleImpl)]
pub trait RewardsModule {

    #[module(EventsModuleImpl)]
    fn events(&self) -> EventsModuleImpl<T, BigInt, BigUint>;

    #[module(NodeConfigModuleImpl)]
    fn node_config(&self) -> NodeConfigModuleImpl<T, BigInt, BigUint>;

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(UserStakeModuleImpl)]
    fn user_stake(&self) -> UserStakeModuleImpl<T, BigInt, BigUint>;

    #[module(NodeActivationModuleImpl)]
    fn node_activation(&self) -> NodeActivationModuleImpl<T, BigInt, BigUint>;



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
        self.storage_load_cumulated_validator_reward()
    }

    /// The account running the nodes is entitled to (service_fee / NODE_DENOMINATOR) * rewards.
    fn service_fee_reward(&self, tot_rewards: &BigUint) -> BigUint {
        let mut node_rewards = tot_rewards * &self.node_config().get_service_fee();
        node_rewards /= BigUint::from(SERVICE_FEE_DENOMINATOR);
        node_rewards
    }

    /// Does not update storage, only returns the updated user data object.
    fn load_user_data_update_rewards(&self, user_id: usize) -> UserRewardData<BigUint> {
        let mut user_data = self.user_data().load_user_data(user_id);

        // new rewards are what was added since the last time rewards were computed
        let tot_cumul_rewards = self.get_total_cumulated_rewards();
        let tot_new_rewards = &tot_cumul_rewards - &user_data.reward_checkpoint;
        if tot_new_rewards == 0 {
            return user_data; // nothing happened since the last claim
        }

        // the owner is entitled to: new rewards * service_fee / NODE_DENOMINATOR
        let service_fee = self.service_fee_reward(&tot_new_rewards);
        
        // update node rewards, if applicable
        if user_id == OWNER_USER_ID {
            user_data.unclaimed_rewards += &service_fee;
        }

        // update delegator rewards, if applicable
        if user_data.active_stake > 0 {
            // delegator reward is:
            // total new rewards * (1 - service_fee / NODE_DENOMINATOR) * user stake / total stake
            let mut delegator_new_rewards = tot_new_rewards - service_fee;
            delegator_new_rewards *= &user_data.active_stake;
            delegator_new_rewards /= &self.user_data().get_user_stake_of_type(USER_STAKE_TOTALS_ID, UserStakeState::Active);
            user_data.unclaimed_rewards += &delegator_new_rewards;
        }

        // update user data checkpoint
        user_data.reward_checkpoint = tot_cumul_rewards;

        user_data
    }

    /// Computes rewards for all delegators and the node.
    /// Updates storage.
    /// Could cost a lot of gas.
    #[endpoint(computeAllRewards)]
    fn compute_all_rewards(&self) {
        let num_nodes = self.user_data().get_num_users();
        let mut sum_unclaimed = BigUint::zero();

        // user 1 is the node and from 2 on are the other delegators,
        // but _load_user_data_update_rewards handles them all
        for user_id in 1..(num_nodes+1) {
            let user_data = self.load_user_data_update_rewards(user_id);
            self.user_data().store_user_data(user_id, &user_data);
            sum_unclaimed += user_data.unclaimed_rewards;
        }

        // divisions are inexact so a small remainder can remain after distributing rewards
        // give it to the validator user, to keep things clear
        let remainder = self.get_total_cumulated_rewards() - sum_unclaimed - self.get_sent_rewards();
        if remainder > 0 {
            let mut node_unclaimed = self.user_data().get_user_rew_unclaimed(OWNER_USER_ID);
            node_unclaimed += &remainder;
            self.user_data().set_user_rew_unclaimed(OWNER_USER_ID, &node_unclaimed);
        }
    }

    /// Yields how much a user is able to claim in rewards at the present time.
    /// Does not update storage.
    #[view(getClaimableRewards)]
    fn get_claimable_rewards(&self, user: Address) -> BigUint {
        let user_id = self.user_data().get_user_id(&user);
        if user_id == 0 {
            return BigUint::zero()
        }

        let user_data = self.load_user_data_update_rewards(user_id);
        user_data.unclaimed_rewards
    }

    /// Utility readonly function to check how many unclaimed rewards currently reside in the contract.
    #[view(getTotalUnclaimedRewards)]
    fn get_total_unclaimed_rewards(&self) -> BigUint {
        let num_nodes = self.user_data().get_num_users();
        let mut sum_unclaimed = BigUint::zero();
        
        for user_id in 1..(num_nodes+1) {
            let user_data = self.load_user_data_update_rewards(user_id);
            sum_unclaimed += user_data.unclaimed_rewards;
        }

        sum_unclaimed
    }

    /// Retrieve those rewards to which the caller is entitled.
    /// Will send:
    /// - new rewards
    /// - rewards that were previously computed but not sent
    #[endpoint(claimRewards)]
    fn claim_rewards(&self) -> Result<(), SCError> {
        let caller = self.get_caller();
        let user_id = self.user_data().get_user_id(&caller);
        if user_id == 0 {
            return sc_error!("unknown caller")
        }

        let mut user_data = self.load_user_data_update_rewards(user_id);

        if user_data.unclaimed_rewards > 0 {
            self.send_rewards(&caller, &user_data.unclaimed_rewards);
            user_data.unclaimed_rewards = BigUint::zero();
        }

        self.user_data().store_user_data(user_id, &user_data);

        Ok(())
    }

    fn send_rewards(&self, to: &Address, amount: &BigUint) {
        // send funds
        self.send_tx(to, amount, "delegation claim");

        // increment globally sent funds
        let mut sent_rewards = self.get_sent_rewards();
        sent_rewards += amount;
        self.set_sent_rewards(&sent_rewards);
    }

}