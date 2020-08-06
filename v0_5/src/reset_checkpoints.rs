
use crate::rewards::*;
use crate::settings::*;
use user_fund_storage::user_data::*;
use crate::global_checkpoint::*;

imports!();

pub static STOP_AT_GASLIMIT: i64 = 1000000;

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
    fn get_global_check_point(&self) -> Option<GlobalCheckpoint<BigUint>>;

    #[storage_set("global_check_point")]
    fn set_global_check_point(&self, user: Option<GlobalCheckpoint<BigUint>>);

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
        if !self.get_global_check_point_in_progress() {
            return sc_error!("compute all rewards is enabled only if checkpoint is in progress")
        }

        let mut opt_global_checkpoint = self.get_global_check_point();
        if let Some(curr_global_checkpoint) = &mut opt_global_checkpoint {
            let num_nodes = self.user_data().get_num_users();
            let mut sum_unclaimed = curr_global_checkpoint.sum_unclaimed.clone();

            for user_id in curr_global_checkpoint.last_id..(num_nodes+1) {
                if self.get_gas_left() < STOP_AT_GASLIMIT {
                    curr_global_checkpoint.last_id = user_id + 1;
                    curr_global_checkpoint.sum_unclaimed = sum_unclaimed;
                    self.set_global_check_point(Some(curr_global_checkpoint.clone()));
                    return Ok(());
                }

                let user_data = self.rewards().load_updated_user_rewards(user_id);
                self.rewards().store_user_reward_data(user_id, &user_data);
                sum_unclaimed += user_data.unclaimed_rewards;
            }

            // divisions are inexact so a small remainder can remain after distributing rewards
            // give it to the validator user, to keep things clear
            curr_global_checkpoint.sum_unclaimed = sum_unclaimed.clone();
            let remainder = self.rewards().get_total_cumulated_rewards() - sum_unclaimed - self.rewards().get_sent_rewards();
            if remainder > 0 {
                let mut node_unclaimed = self.rewards().get_user_rew_unclaimed(OWNER_USER_ID);
                node_unclaimed += &remainder;
                self.rewards().set_user_rew_unclaimed(OWNER_USER_ID, &node_unclaimed);
            }

            curr_global_checkpoint.last_id = num_nodes+1;
            curr_global_checkpoint.finished = true;
            self.set_global_check_point_in_progress(false);
            self.set_global_check_point(Some(curr_global_checkpoint.clone()));

            Ok(())            

        } else {
            return sc_error!("impossible error")
        }
    }

    fn start_checkpoint_compute(&self, total_cap: BigUint) {
        let opt_global_checkpoint = Some(GlobalCheckpoint {
            total_delegation_cap: total_cap,
            finished:             false,
            last_id:              1,
            sum_unclaimed:        BigUint::zero(),
        });

        self.set_global_check_point_in_progress(true);
        self.set_global_check_point(opt_global_checkpoint);
    }

    // total delegation cap can be modified by owner only, it will recalculate and set the checkpoint for all the delegators
    // can be called only by owner - it might be used only in accordance with the delegators
    #[endpoint(modifyTotalDelegationCap)]
    fn modify_total_delegation_cap(&self, new_total_cap: BigUint) -> SCResult<()> {
        if !self.settings().owner_called() {
            return sc_error!("caller not allowed to modify delegation cap");
        }

        if new_total_cap == self.settings().get_total_delegation_cap() {
            return Ok(())
        }
        if self.get_global_check_point_in_progress() {
            return sc_error!("cannot modify total delegation cap when last is in progress");
        }

        self.start_checkpoint_compute(new_total_cap);

        Ok(())
    }
}