
use crate::rewards::*;
use crate::settings::*;
use user_fund_storage::user_data::*;
use user_fund_storage::fund_transf_module::*;
use user_fund_storage::fund_view_module::*;
use user_fund_storage::types::*;
use crate::global_checkpoint::*;
use core::cmp::Ordering;

imports!();

pub static STOP_AT_GASLIMIT: i64 = 1000000;

pub const COMPUTATION_DONE: bool = false;
pub const MORE_TO_COMPUTE: bool = true;

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


    #[view(getInterruptedComputation)]
    #[storage_get("interrupted_computation")]
    fn get_interrupted_computation(&self) -> ExtendedComputation<BigUint>;

    #[storage_set("interrupted_computation")]
    fn set_interrupted_computation(&self, ec: &ExtendedComputation<BigUint>);

    #[view(isInterruptedComputation)]
    fn is_interrupted_computation(&self) -> bool {
        // TODO: make this pattern into an attribute just like storage_get/storage_set in elrond_wasm
        // something like storage_is_empty
        self.storage_load_len(&b"interrupted_computation"[..]) > 0
    }

    // #[storage_get("global_check_point")]
    // fn get_global_check_point(&self) -> Option<GlobalCheckpoint<BigUint>>;

    // #[storage_set("global_check_point")]
    // fn set_global_check_point(&self, user: Option<GlobalCheckpoint<BigUint>>);

    // #[storage_get("swap_check_point")]
    // fn get_swap_check_point(&self) -> SwapCheckpoint<BigUint>;

    // #[storage_set("swap_check_point")]
    // fn set_swap_check_point(&self, user: SwapCheckpoint<BigUint>);

    // #[storage_get("global_check_point_in_progress")]
    // fn get_global_check_point_in_progress(&self) -> bool;

    // #[storage_set("global_check_point_in_progress")]
    // fn set_global_check_point_in_progress(&self, in_progress: bool);

    // #[storage_get("swap_in_progress")]
    // fn get_swap_in_progress(&self) -> bool;

    // #[storage_set("swap_in_progress")]
    // fn set_swap_in_progress(&self, in_progress: bool);

    #[endpoint(continueComputation)]
    fn continue_computation_endpoint(&self) -> SCResult<bool> {
        let ec = self.get_interrupted_computation();
        self.continue_computation(ec)
    }

    fn continue_computation(&self, mut ec: ExtendedComputation<BigUint>) -> SCResult<bool> {
        while !ec.is_empty() {
            let (out_of_gas, new_ic) = self.perform_interrupted_computation_step(ec);
            ec = new_ic;
            if out_of_gas {
                self.set_interrupted_computation(&ec);
                return Ok(MORE_TO_COMPUTE);
            }
        }

        self.set_interrupted_computation(&ec); 
        Ok(COMPUTATION_DONE)
    }

    fn perform_interrupted_computation_step(&self, ec: ExtendedComputation<BigUint>) -> (bool, ExtendedComputation<BigUint>) {
        match ec.computation_type {
            ComputationType::ChangeServiceFee => {
                if let ComputationStep::ComputeAllRewards(data) = ec.step {
                    if let Some(more_computation) = self.compute_all_rewards(data) {
                        return (true, ExtendedComputation{
                            step: ComputationStep::ComputeAllRewards(more_computation),
                            ..ec
                        })
                    } else {
                        self.settings().set_service_fee(self.settings().get_new_service_fee());
                    }
                }
                
                (false, ExtendedComputation{
                    step: ComputationStep::None,
                    ..ec
                })
            },
            ComputationType::ModifyTotalDelegationCap => {
                match ec.step {
                    ComputationStep::ComputeAllRewards(data) => {
                        if let Some(more_computation) = self.compute_all_rewards(data) {
                            (true, ExtendedComputation{
                                step: ComputationStep::ComputeAllRewards(more_computation),
                                ..ec
                            })
                        } else {
                            (false, ExtendedComputation{
                                step: ComputationStep::SwapUnstakedToDeferredPayment,
                                ..ec
                            })
                        }
                    },
                    ComputationStep::SwapWaitingToActive => {
                        let (_, remaining) = self.fund_transf_module().swap_waiting_to_active(
                            &ec.remaining_swap_waiting_to_active,
                            || self.get_gas_left() < STOP_AT_GASLIMIT
                        );
                        if remaining > 0 {
                            (true, ExtendedComputation{
                                remaining_swap_waiting_to_active: remaining,
                                ..ec
                            })
                        } else {
                            (false, ExtendedComputation{
                                remaining_swap_waiting_to_active: BigUint::zero(),
                                step: ComputationStep::SwapUnstakedToDeferredPayment,
                                ..ec
                            })
                        }
                    },
                    ComputationStep::SwapUnstakedToDeferredPayment => {
                        let remaining = self.fund_transf_module().swap_unstaked_to_deferred_payment(
                            &ec.remaining_swap_unstaked_to_def_p,
                            || self.get_gas_left() < STOP_AT_GASLIMIT
                        );
                        if remaining > 0 {
                            (true, ExtendedComputation{
                                remaining_swap_unstaked_to_def_p: remaining,
                                ..ec
                            })
                        } else {
                            (false, ExtendedComputation{
                                remaining_swap_unstaked_to_def_p: BigUint::zero(),
                                step: ComputationStep::SwapActiveToDeferredPayment,
                                ..ec
                            })
                        }
                    },
                    ComputationStep::SwapActiveToDeferredPayment => {
                        let remaining = self.fund_transf_module().swap_active_to_deferred_payment(
                            &ec.remaining_swap_active_to_def_p,
                            || self.get_gas_left() < STOP_AT_GASLIMIT
                        );
                        if remaining > 0 {
                            (true, ExtendedComputation{
                                remaining_swap_active_to_def_p: remaining,
                                ..ec
                            })
                        } else {
                            (false, ExtendedComputation{
                                remaining_swap_active_to_def_p: BigUint::zero(),
                                step: ComputationStep::None,
                                ..ec
                            })
                        }
                    },
                    ComputationStep::None => (false, ec),               
                }
            }
        }
    }


    /// when there is a change of the base cap from where the rewards are computed
    /// the checkpoints must be reset for all the delegators
    /// this process might be longer then one block - reaching the gaslimit
    /// thus will do it by saving where it left before reaching out of gas
    /// no change in the delegators total cap is allowed until all the checkpoints are not recalculated
    /// 
    /// returns something if not done computing
    fn compute_all_rewards(&self, mut data: ComputeAllRewardsData<BigUint>) -> Option<ComputeAllRewardsData<BigUint>> {
        // if epoch changed, computation must be started from scratch
        // TODO: base this on reward checkpoint instead of epoch to fix edge case
        let curr_epoch = self.get_block_epoch();
        if data.epoch != curr_epoch {
            data.last_id = 0;
            data.sum_unclaimed = BigUint::zero();
            data.epoch = curr_epoch;
        }

        let num_nodes = self.user_data().get_num_users();

        while data.last_id <= num_nodes {
            if self.get_gas_left() < STOP_AT_GASLIMIT {
                return Some(data);
            }

            let current_user_id = data.last_id + 1;
            let user_data = self.rewards().load_updated_user_rewards(current_user_id);
            self.rewards().store_user_reward_data(current_user_id, &user_data);
            data.sum_unclaimed += user_data.unclaimed_rewards;
            data.last_id = current_user_id;
        }

        // divisions are inexact so a small remainder can remain after distributing rewards
        // give it to the owner, to keep things clear
        let remainder = &self.rewards().get_total_cumulated_rewards() - &data.sum_unclaimed - self.rewards().get_sent_rewards();
        if remainder > 0 {
            let mut node_unclaimed = self.rewards().get_user_rew_unclaimed(OWNER_USER_ID);
            node_unclaimed += &remainder;
            self.rewards().set_user_rew_unclaimed(OWNER_USER_ID, &node_unclaimed);
        }

        None
    }

    // might be called only if checkpoint is finished, but globak checkpoint is still in progress as the swap
    // of user funds from waiting to staking, or from staking to unstaked to deffered payment must be made before restarting
    // all the other functions. 
    // As this process might be long as well - swapping multiple funds - the function can be called multiple times to resolve all
    // #[endpoint(endCheckpointCompute)]
    // fn end_checkpoint_compute(&self) -> SCResult<BigUint> {
    //     if !self.get_global_check_point_in_progress() {
    //         return sc_error!("cannot call end checkpoint as checkpoint reset is not in progress");
    //     }
    //     if self.get_swap_in_progress() {
    //         return sc_error!("cannot call end checkpoint compute as swap is not in progress");
    //     }

    //     let opt_global_checkpoint = self.get_global_check_point();
    //     if let Some(curr_global_checkpoint) = opt_global_checkpoint {
    //         if curr_global_checkpoint.last_id != 0 {
    //             return sc_error!("cannot call end checkpoint as compute all rewards has not finished");
    //         }

    //         let old_delegation_cap = self.settings().get_total_delegation_cap();
    //         self.settings().set_total_delegation_cap(curr_global_checkpoint.total_delegation_cap.clone());

    //         if curr_global_checkpoint.total_delegation_cap < old_delegation_cap {
    //             // move active to unstake to deferred
    //             let amount_to_swap = &old_delegation_cap - &curr_global_checkpoint.total_delegation_cap;
    //             let total_unstaked = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::UnStaked);

    //             let amount_to_unstake = core::cmp::min(BigUint::zero(), amount_to_swap.clone() - total_unstaked);
    //             let (_, remaining) = self.fund_transf_module().swap_active_to_unstaked(
    //                 &amount_to_unstake,
    //                 || self.get_gas_left() < STOP_AT_GASLIMIT
    //             );
    //             if remaining > 0 {
    //                 self.save_swapping_checkpoint(FundType::Active, remaining.clone(), amount_to_swap);
    //                 return Ok(remaining.clone());
    //             }

    //             let remaining_for_defer = self.fund_transf_module().swap_unstaked_to_deferred_payment(
    //                 &amount_to_swap,
    //                 || self.get_gas_left() < STOP_AT_GASLIMIT
    //             );
    //             if remaining_for_defer > 0 {
    //                 self.save_swapping_checkpoint(FundType::UnStaked, remaining_for_defer.clone(), amount_to_swap);
    //                 return Ok(remaining_for_defer.clone());
    //             }
    //         } else if curr_global_checkpoint.total_delegation_cap > old_delegation_cap {
    //             // move waiting to active
    //             let amount_to_swap = curr_global_checkpoint.total_delegation_cap.clone() - old_delegation_cap.clone();
    //             let (_, remaining) = self.fund_transf_module().swap_waiting_to_active(
    //                 &amount_to_swap,
    //                 || self.get_gas_left() < STOP_AT_GASLIMIT
    //             );
    //             if remaining > 0 {
    //                 self.save_swapping_checkpoint(FundType::Waiting, remaining.clone(), amount_to_swap);
    //                 return Ok(remaining.clone());
    //             }
    //         } else {
    //             self.settings().set_service_fee(self.settings().get_new_service_fee());
    //         }

    //         self.set_swap_in_progress(false);
    //         self.set_global_check_point_in_progress(false);
    //         return Ok(BigUint::zero());

    //     } else {
    //         return sc_error!("impossible error")
    //     }
    // }

    // // continues to swap the pending action
    // #[endpoint(continueSwap)]
    // fn continue_swap(&self) -> SCResult<BigUint> {
    //     if !self.get_swap_in_progress() {
    //         return sc_error!("there is no swap in progress");
    //     }

    //     let mut swap_checkpoint = self.get_swap_check_point();
    //     match swap_checkpoint.f_type {
    //         FundType::Waiting => {
    //             let (_, remaining) = self.fund_transf_module().swap_waiting_to_active(
    //                 &swap_checkpoint.remaining,
    //                 || self.get_gas_left() < STOP_AT_GASLIMIT
    //             );
    //             if remaining > 0 {
    //                 swap_checkpoint.remaining = remaining.clone();
    //                 self.set_swap_check_point(swap_checkpoint);
    //                 return Ok(remaining);
    //             }
    //         },
    //         FundType::Active => {
    //             let (_, remaining) = self.fund_transf_module().swap_active_to_unstaked(
    //                 &swap_checkpoint.remaining,
    //                 || self.get_gas_left() < STOP_AT_GASLIMIT
    //             );
    //             if remaining > 0 {
    //                 swap_checkpoint.remaining = remaining.clone();
    //                 self.set_swap_check_point(swap_checkpoint);
    //                 return Ok(remaining.clone());
    //             }

    //             let remaining_for_defer = self.fund_transf_module().swap_unstaked_to_deferred_payment(
    //                 &swap_checkpoint.initial,
    //                 || self.get_gas_left() < STOP_AT_GASLIMIT
    //             );
    //             if remaining_for_defer > 0 {
    //                 swap_checkpoint.remaining = remaining_for_defer.clone();
    //                 self.set_swap_check_point(swap_checkpoint);
    //                 return Ok(remaining_for_defer.clone())
    //             }
    //         },
    //         FundType::UnStaked => {
    //             let remaining = self.fund_transf_module().swap_unstaked_to_deferred_payment(
    //                 &swap_checkpoint.remaining,
    //                 || self.get_gas_left() < STOP_AT_GASLIMIT
    //             );
    //             if remaining > 0 {
    //                 swap_checkpoint.remaining = remaining.clone();
    //                 self.set_swap_check_point(swap_checkpoint);
    //                 return Ok(remaining);
    //             }
    //         },
    //         _ => return sc_error!("invalid fund type, impossible error"),
    //     }

    //     self.set_global_check_point_in_progress(false);
    //     self.set_swap_in_progress(false);
    //     Ok(BigUint::zero())
    // }

    // fn save_swapping_checkpoint(&self, swap_initial_type: FundType, remaining: BigUint, start_amount: BigUint) {
    //     let swap_checkpoint = SwapCheckpoint{
    //         initial:   start_amount,
    //         remaining: remaining,
    //         f_type:    swap_initial_type,
    //     };
    //     self.set_swap_check_point(swap_checkpoint);
    //     self.set_swap_in_progress(true);
    // }

    // fn start_checkpoint_compute(&self, total_cap: BigUint, total_to_swap: BigUint) {
    //     let opt_global_checkpoint = Some(GlobalCheckpoint {
    //         total_delegation_cap: total_cap,
    //         last_id:              1,
    //         sum_unclaimed:        BigUint::zero(),
    //         total_to_swap:        total_to_swap,
    //         epoch:                self.get_block_epoch(),
    //     });

    //     self.set_global_check_point_in_progress(true);
    //     self.set_global_check_point(opt_global_checkpoint);
    // }

    // total delegation cap can be modified by owner only, it will recalculate and set the checkpoint for all the delegators
    // can be called only by owner - it might be used only in accordance with the delegators
    #[endpoint(modifyTotalDelegationCap)]
    fn modify_total_delegation_cap(&self, new_total_cap: BigUint) -> SCResult<bool> {
        require!(self.settings().owner_called(),
            "only owner allowed to modify delegation cap");

        require!(!self.is_interrupted_computation(),
            "cannot modify total delegation cap when last is in progress");

        let curr_delegation_cap = self.settings().get_total_delegation_cap();
        let total_waiting = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Waiting);
        let total_active = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active);
        let total_unstaked = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::UnStaked);

        let max_available = &total_active + &total_waiting;
        require!(new_total_cap <= max_available,
            "new delegation cap must be less or equal to total active + waiting");

        let ec = match new_total_cap.cmp(&curr_delegation_cap) {
            Ordering::Equal => { // nothing changes
                return Ok(COMPUTATION_DONE)
            },
            Ordering::Greater => { // cap increases
                require!(total_unstaked == 0,
                    "no unstaked funds should be present when increasing delegation cap");

                let swap_amount = &new_total_cap - &curr_delegation_cap;
                ExtendedComputation{
                    total_delegation_cap: new_total_cap,
                    remaining_swap_waiting_to_active: swap_amount,
                    remaining_swap_active_to_def_p: BigUint::zero(),
                    remaining_swap_unstaked_to_def_p: BigUint::zero(),
                    computation_type: ComputationType::ModifyTotalDelegationCap,
                    step: ComputationStep::ComputeAllRewards(ComputeAllRewardsData::new(self.get_block_epoch())),
                }
            },
            Ordering::Less => { // cap decreases
                let swap_amount = &curr_delegation_cap - &new_total_cap;
                require!(swap_amount <= self.rewards().total_unprotected(),
                    "not enough funds in contract to pay those who are forced unstaked");
                
                let swap_unstaked_to_def_p: BigUint;
                let swap_active_to_def_p: BigUint;
                if total_unstaked >= swap_amount {
                    // only unstaked -> deferred payment will happen
                    swap_active_to_def_p = BigUint::zero();
                    swap_unstaked_to_def_p = swap_amount;
                } else {
                    // first unstaked -> deferred payment happens, then active -> deferred payment
                    swap_active_to_def_p = &swap_amount - &total_unstaked;
                    swap_unstaked_to_def_p = total_unstaked;
                }
                
                ExtendedComputation{
                    total_delegation_cap: new_total_cap,
                    remaining_swap_waiting_to_active: BigUint::zero(),
                    remaining_swap_active_to_def_p: swap_active_to_def_p,
                    remaining_swap_unstaked_to_def_p: swap_unstaked_to_def_p,
                    computation_type: ComputationType::ModifyTotalDelegationCap,
                    step: ComputationStep::ComputeAllRewards(ComputeAllRewardsData::new(self.get_block_epoch())),
                }
            }
        };

        self.continue_computation(ec)
    }
}
