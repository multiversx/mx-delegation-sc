
use crate::rewards::*;
use crate::settings::*;
use user_fund_storage::user_data::*;
use user_fund_storage::fund_transf_module::*;
use user_fund_storage::fund_view_module::*;
use user_fund_storage::types::*;
use crate::global_checkpoint::*;

imports!();

pub static STOP_AT_GASLIMIT: i64 = 1000000;

#[elrond_wasm_derive::module(ResetCheckpointsModuleImpl)]
pub trait ResetCheckpointsModule {
    
    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(FundTransformationsModuleImpl)]
    fn fund_transf_module(&self) -> FundTransformationsModuleImpl<T, BigInt, BigUint>;

    #[module(FundViewModuleImpl)]
    fn fund_view_module(&self) -> FundViewModuleImpl<T, BigInt, BigUint>;

    #[module(RewardsModuleImpl)]
    fn rewards(&self) -> RewardsModuleImpl<T, BigInt, BigUint>;

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    #[storage_get("global_check_point")]
    fn get_global_check_point(&self) -> Option<GlobalCheckpoint<BigUint>>;

    #[storage_set("global_check_point")]
    fn set_global_check_point(&self, user: Option<GlobalCheckpoint<BigUint>>);

    #[storage_get("swap_check_point")]
    fn get_swap_check_point(&self) -> SwapCheckpoint<BigUint>;

    #[storage_set("swap_check_point")]
    fn set_swap_check_point(&self, user: SwapCheckpoint<BigUint>);

    #[storage_get("global_check_point_in_progress")]
    fn get_global_check_point_in_progress(&self) -> bool;

    #[storage_set("global_check_point_in_progress")]
    fn set_global_check_point_in_progress(&self, in_progress: bool);

    #[storage_get("swap_in_progress")]
    fn get_swap_in_progress(&self) -> bool;

    #[storage_set("swap_in_progress")]
    fn set_swap_in_progress(&self, in_progress: bool);

    /// when there is a change of the base cap from where the rewards are computed
    /// the checkpoints must be reset for all the delegators
    /// this process might be longer then one block - reaching the gaslimit
    /// thus will do it by saving where it left before reaching out of gas
    /// no change in the delegators total cap is allowed until all the checkpoints are not recalculated
    #[endpoint(computeAllRewards)]
    fn compute_all_rewards(&self) -> SCResult<usize> {
        if !self.get_global_check_point_in_progress() {
            return sc_error!("compute all rewards is enabled only if checkpoint is in progress")
        }

        let mut opt_global_checkpoint = self.get_global_check_point();
        if let Some(curr_global_checkpoint) = &mut opt_global_checkpoint {
            if curr_global_checkpoint.last_id == 0 {
                return Ok(0);
            }

            let num_nodes = self.user_data().get_num_users();
            let mut sum_unclaimed = curr_global_checkpoint.sum_unclaimed.clone();

            for user_id in curr_global_checkpoint.last_id..(num_nodes+1) {
                if self.get_gas_left() < STOP_AT_GASLIMIT {
                    curr_global_checkpoint.last_id = user_id;
                    curr_global_checkpoint.sum_unclaimed = sum_unclaimed;
                    self.set_global_check_point(Some(curr_global_checkpoint.clone()));
                    return Ok(user_id);
                }

                let user_data = self.rewards().load_updated_user_rewards(user_id);
                self.rewards().store_user_reward_data(user_id, &user_data);
                sum_unclaimed += user_data.unclaimed_rewards;
            }

            // one epoch past thus the global checkpoint computation has to start from the first user ID
            let curr_epoch = self.get_block_epoch();
            if curr_global_checkpoint.epoch != curr_epoch {
                curr_global_checkpoint.epoch = curr_epoch;
                curr_global_checkpoint.last_id = 1;
                curr_global_checkpoint.sum_unclaimed = BigUint::zero();
                self.set_global_check_point(Some(curr_global_checkpoint.clone()));
                return Ok(1)
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

            curr_global_checkpoint.last_id = 0;
            self.set_global_check_point(Some(curr_global_checkpoint.clone()));
            self.set_swap_in_progress(true);

            Ok(0)            

        } else {
            return sc_error!("impossible error")
        }
    }

    // might be called only if checkpoint is finished, but globak checkpoint is still in progress as the swap
    // of user funds from waiting to staking, or from staking to unstaked to deffered payment must be made before restarting
    // all the other functions. 
    // As this process might be long as well - swapping multiple funds - the function can be called multiple times to resolve all
    #[endpoint(endCheckpointCompute)]
    fn end_checkpoint_compute(&self) -> SCResult<BigUint> {
        if !self.get_global_check_point_in_progress() {
            return sc_error!("cannot call end checkpoint as checkpoint reset is not in progress");
        }
        if !self.get_swap_in_progress() {
            return sc_error!("cannot call end checkpoint compute as swap is not in progress");
        }

        let opt_global_checkpoint = self.get_global_check_point();
        if let Some(curr_global_checkpoint) = opt_global_checkpoint {
            if curr_global_checkpoint.last_id != 0 {
                return sc_error!("cannot call end checkpoint as compute all rewards has not finished");
            }

            let old_delegation_cap = self.settings().get_total_delegation_cap();
            self.settings().set_total_delegation_cap(curr_global_checkpoint.total_delegation_cap.clone());

            if curr_global_checkpoint.total_delegation_cap < old_delegation_cap {
                // move active to unstake to deferred
                let amount_to_swap = &old_delegation_cap - &curr_global_checkpoint.total_delegation_cap;

                let (_, remaining) = self.fund_transf_module().swap_active_to_unstaked(
                    &amount_to_swap,
                    || self.get_gas_left() < STOP_AT_GASLIMIT
                );
                if remaining > 0 {
                    self.save_swapping_checkpoint(FundType::Active, remaining.clone(), amount_to_swap);
                    return Ok(remaining.clone());
                }

                let remaining_for_defer = self.fund_transf_module().swap_unstaked_to_deferred_payment(
                    &amount_to_swap,
                    || self.get_gas_left() < STOP_AT_GASLIMIT
                );
                if remaining_for_defer > 0 {
                    self.save_swapping_checkpoint(FundType::UnStaked, remaining_for_defer.clone(), amount_to_swap);
                    return Ok(remaining_for_defer.clone());
                }
            } else if curr_global_checkpoint.total_delegation_cap > old_delegation_cap {
                // move waiting to active
                let amount_to_swap = curr_global_checkpoint.total_delegation_cap.clone() - old_delegation_cap.clone();
                let (_, remaining) = self.fund_transf_module().swap_waiting_to_active(
                    &amount_to_swap,
                    || self.get_gas_left() < STOP_AT_GASLIMIT
                );
                if remaining > 0 {
                    self.save_swapping_checkpoint(FundType::Waiting, remaining.clone(), amount_to_swap);
                    return Ok(remaining.clone());
                }
            } else {
                self.settings().set_service_fee(self.settings().get_new_service_fee());
            }

            self.set_swap_in_progress(false);
            self.set_global_check_point_in_progress(false);
            return Ok(BigUint::zero());

        } else {
            return sc_error!("impossible error")
        }
    }

    fn save_swapping_checkpoint(&self, swap_initial_type: FundType, remaining: BigUint, start_amount: BigUint) {
        let swap_checkpoint = SwapCheckpoint{
            initial:   start_amount,
            remaining: remaining,
            f_type:    swap_initial_type,
        };
        self.set_swap_check_point(swap_checkpoint);
    }

    fn start_checkpoint_compute(&self, total_cap: BigUint, total_to_swap: BigUint) {
        let opt_global_checkpoint = Some(GlobalCheckpoint {
            total_delegation_cap: total_cap,
            last_id:              1,
            sum_unclaimed:        BigUint::zero(),
            total_to_swap:        total_to_swap,
            epoch:                self.get_block_epoch(),
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

        let curr_delegation_cap = self.settings().get_total_delegation_cap();
        if new_total_cap == curr_delegation_cap {
            return Ok(())
        }
        if self.get_global_check_point_in_progress() {
            return sc_error!("cannot modify total delegation cap when last is in progress");
        }

        let total_waiting = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Waiting);
        let total_active = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active);
        let max_available = total_active.clone() + total_waiting.clone();
        if new_total_cap > max_available {
            return sc_error!("cannot add to delegation cap more then max available");
        }

        let total_to_swap : BigUint;
        if curr_delegation_cap > new_total_cap {
            total_to_swap = curr_delegation_cap - new_total_cap.clone();
            if total_to_swap < self.rewards().total_unprotected() {
                return sc_error!("not enough funds in contract to pay those who are forced unstaked");
            }
        } else {
            total_to_swap = new_total_cap.clone() - curr_delegation_cap;
        }

        self.start_checkpoint_compute(new_total_cap, total_to_swap);

        Ok(())
    }
}
