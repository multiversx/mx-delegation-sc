
use crate::rewards::*;
use crate::settings::*;
use user_fund_storage::user_data::*;
use crate::global_checkpoint::*;

imports!();

pub static STOP_AT_GASLIMIT: u64 = 1000000;

/// Contains endpoints for staking/withdrawing stake.
#[elrond_wasm_derive::module(ResetCheckpointsModuleImpl)]
pub trait ResetCheckpointsModule {
    
    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(RewardsModuleImpl)]
    fn rewards(&self) -> RewardsModuleImpl<T, BigInt, BigUint>;

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    #[storage_get("global_check_point")]
    fn get_global_check_point(&self) -> Option<GlobalCheckpoint>;

    #[storage_set("global_check_point")]
    fn set_global_check_point(&self, user: Option<GlobalCheckpoint>);

    #[storage_get("global_check_point_in_progress")]
    fn get_global_check_point_in_progress(&self) -> bool;

    #[storage_set("global_check_point_in_progress")]
    fn set_global_check_point_in_progress(&self, in_progress: bool);

    /// when there is a change of the base cap from where the rewards are computed
    /// the checkpoints must be reset for all the delegators
    /// this process might be longer then one block - reaching the gaslimit
    /// thus will do it by saving where it left before reaching out of gas
    /// no change in the delegators total cap is allowed until all the checkpoints are not recalculated
    #[endpoint(computeAllRewards)]
    fn compute_all_rewards(&self) -> SCResult<()> {

        let num_nodes = self.user_data().get_num_users();
        let mut sum_unclaimed = BigUint::zero();

        // user 1 is the node and from 2 on are the other delegators,
        // but load_updated_user_rewards works in all cases
        for user_id in 1..(num_nodes+1) {
            let user_data = self.rewards().load_updated_user_rewards(user_id);
            self.rewards().store_user_reward_data(user_id, &user_data);
            sum_unclaimed += user_data.unclaimed_rewards;
        }

        // divisions are inexact so a small remainder can remain after distributing rewards
        // give it to the validator user, to keep things clear
        let remainder = self.rewards().get_total_cumulated_rewards() - sum_unclaimed - self.rewards().get_sent_rewards();
        if remainder > 0 {
            let mut node_unclaimed = self.rewards().get_user_rew_unclaimed(OWNER_USER_ID);
            node_unclaimed += &remainder;
            self.rewards().set_user_rew_unclaimed(OWNER_USER_ID, &node_unclaimed);
        }
        Ok(())
    }

    fn start_checkpoint_compute(&self, total_cap: BigUint) {
        let mut opt_global_checkpoint = self.get_global_check_point();
        if opt_global_checkpoint.is_none() {
            opt_global_checkpoint = Some(GlobalCheckpoint {
                totalDelegationCap: total_cap,
                finished:           false,
                lastID:             1,
            });

            self.set_global_check_point_in_progress(true);
            self.set_global_check_point(opt_global_checkpoint);
            return
        }

        if let Some(global_checkpoint) = &mut opt_global_checkpoint {
            if global_checkpoint.totalDelegationCap == total_cap && global_checkpoint.finished == true {
                return
            }
            global_checkpoint.totalDelegationCap = total_cap;
            global_checkpoint.finished = false;
            global_checkpoint.lastID = 1;

            self.set_global_check_point_in_progress(true);
            self.set_global_check_point(opt_global_checkpoint);
        }
    }

    fn end_checkpoint_compute(&self, global_checkpoint: GlobalCheckpoint) {
        global_checkpoint.finished = true;

        self.set_global_check_point_in_progress(false);
        self.set_global_check_point(Some(global_checkpoint));
    }

    #[endpoint(modifyTotalDelegationCap)]
    fn modify_total_delegation_cap(&self, new_total_cap: BigUint) -> SCResult<()> {
        if !self.settings().owner_called() {
            return sc_error!("caller not allowed to modify delegation cap")
        }

        if new_total_cap == self.settings().get_total_delegation_cap() {
            return Ok(())
        }

        self.start_checkpoint_compute(new_total_cap);

        Ok(())
    }
}