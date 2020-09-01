
use crate::events::*;
use crate::pause::*;
use crate::rewards::*;
use crate::settings::*;
use crate::reset_checkpoints::*;
use crate::user_stake::*;
use user_fund_storage::user_data::*;
use user_fund_storage::fund_transf_module::*;
use user_fund_storage::fund_view_module::*;
use user_fund_storage::types::*;

imports!();

#[elrond_wasm_derive::module(UserUnStakeModuleImpl)]
pub trait UserUnStakeModule {

    #[module(EventsModuleImpl)]
    fn events(&self) -> EventsModuleImpl<T, BigInt, BigUint>;

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(FundTransformationsModuleImpl)]
    fn fund_transf_module(&self) -> FundTransformationsModuleImpl<T, BigInt, BigUint>;

    #[module(FundViewModuleImpl)]
    fn fund_view_module(&self) -> FundViewModuleImpl<T, BigInt, BigUint>;

    #[module(UserStakeModuleImpl)]
    fn user_stake(&self) -> UserStakeModuleImpl<T, BigInt, BigUint>;

    #[module(PauseModuleImpl)]
    fn pause(&self) -> PauseModuleImpl<T, BigInt, BigUint>;

    #[module(RewardsModuleImpl)]
    fn rewards(&self) -> RewardsModuleImpl<T, BigInt, BigUint>;

    #[module(ResetCheckpointsModuleImpl)]
    fn reset_checkpoints(&self) -> ResetCheckpointsModuleImpl<T, BigInt, BigUint>;

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    /// unStake - the user will announce that he wants to get out of the contract
    /// selected funds will change from active to inactive, but claimable only after unBond period ends
    #[endpoint(unStake)]
    fn unstake_endpoint(&self, amount: BigUint) -> SCResult<()> {
        if !self.settings().is_unstake_enabled() {
            return sc_error!("unstake is currently disabled");
        }

        require!(!self.reset_checkpoints().is_global_op_in_progress(),
            "unstaking is temporarily paused as checkpoint is reset");
        
        let caller = self.get_caller();
        let unstake_user_id = self.user_data().get_user_id(&caller);
        if unstake_user_id == 0 {
            return sc_error!("only delegators can unstake")
        }

        // check that amount does not exceed existing active stake
        let stake = self.fund_view_module().get_user_stake_of_type(unstake_user_id, FundType::Active);
        if amount > stake {
            return sc_error!("cannot offer more than the user active stake")
        }
        if amount != stake && amount < self.settings().get_minimum_stake() {
            return sc_error!("cannot unstake less than minimum stake")
        }

        self.rewards().compute_one_user_reward(unstake_user_id);

        // convert Active of this user -> UnStaked
        sc_try!(self.fund_transf_module().unstake_transf(unstake_user_id, &amount));

        // convert Waiting from other users -> Active
        let total_waiting = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Waiting);
        if total_waiting == 0 {
            return Ok(())
        }
        let swappable = core::cmp::min(amount, total_waiting);

        sc_try!(self.user_stake().swap_waiting_to_active_compute_rewards(&swappable));

        // convert UnStaked to defered payment
        let mut unstaked_swappable = swappable;
        self.fund_transf_module().swap_unstaked_to_deferred_payment(&mut unstaked_swappable, || false);
        if unstaked_swappable > 0 {
            return sc_error!("error swapping unstaked to deferred payment")
        }

        Ok(())
    }

    #[endpoint(unBond)]
    fn unbond_user(&self) -> SCResult<()> {
        let caller = self.get_caller();
        let caller_id = self.user_data().get_user_id(&caller);
        if caller_id == 0 {
            return sc_error!("unknown caller");
        }

        let n_blocks_before_unbond = self.settings().get_n_blocks_before_unbond();
        let claimed_payments = sc_try!(self.fund_transf_module().claim_all_eligible_deferred_payments(
            caller_id,
            n_blocks_before_unbond
        ));

        let mut amount_to_withdraw = claimed_payments.clone();
        sc_try!(self.fund_transf_module().liquidate_free_stake(caller_id, &mut amount_to_withdraw));
        if amount_to_withdraw > 0 {
            return sc_error!("cannot withdraw more than inactive stake");
        }

        if claimed_payments > 0 {
            // forward payment to seller
            self.send_tx(&caller, &claimed_payments, "payment for stake");
        }

        Ok(())
    }
}
