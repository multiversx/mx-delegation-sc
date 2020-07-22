use crate::fund_type::*;

use crate::events::*;
use crate::pause::*;
use crate::rewards::*;
use crate::settings::*;
use crate::user_data::*;
use crate::fund_transf_module::*;
use crate::fund_view_module::*;

imports!();

/// Deals with stake trade among delegators.
/// Note: each 1 staked ERD can only be traded for 1 unstaked ERD.
#[elrond_wasm_derive::module(StakeSaleModuleImpl)]
pub trait StakeSaleModule {

    #[module(EventsModuleImpl)]
    fn events(&self) -> EventsModuleImpl<T, BigInt, BigUint>;

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(FundTransformationsModuleImpl)]
    fn fund_transf_module(&self) -> FundTransformationsModuleImpl<T, BigInt, BigUint>;

    #[module(FundViewModuleImpl)]
    fn fund_view_module(&self) -> FundViewModuleImpl<T, BigInt, BigUint>;

    #[module(PauseModuleImpl)]
    fn pause(&self) -> PauseModuleImpl<T, BigInt, BigUint>;

    #[module(RewardsModuleImpl)]
    fn rewards(&self) -> RewardsModuleImpl<T, BigInt, BigUint>;

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    /// Creates a stake offer. Overwrites any previous stake offer.
    /// Once a stake offer is up, it can be bought by anyone on a first come first served basis.
    /// Cannot be paused, because this is also part of the unStake mechanism, which the owner cannot veto.
    #[endpoint(unStake)]
    fn unstake_endpoint(&self, amount: BigUint) -> SCResult<()> {
        if !self.settings().is_unstake_enabled() {
            return sc_error!("unstake is currently disabled");
        }
        
        let caller = self.get_caller();
        let unstake_user_id = self.user_data().get_user_id(&caller);
        if unstake_user_id == 0 {
            return sc_error!("only delegators can offer stake for sale")
        }

        // compute rewards 
        self.rewards().compute_all_rewards();

        // check that amount does not exceed existing active stake
        let stake = self.fund_view_module().get_user_stake_of_type(unstake_user_id, FundType::Active);
        if amount > stake {
            return sc_error!("cannot offer more than the user active stake")
        }

        // convert Active -> Unstaked
        sc_try!(self.fund_transf_module().unstake_transf(unstake_user_id, &amount));

        // try to fill the Unstaked stake with Inactive stake in the queue
        sc_try!(self.try_fill_unstaked_from_queue(unstake_user_id, &amount));

        Ok(())
    }

    fn try_fill_unstaked_from_queue(&self, unstake_user_id: usize, amount: &BigUint) -> SCResult<()> {
        let total_inactive = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Inactive);
        if total_inactive == 0 {
            return Ok(());
        }
        let swappable = core::cmp::min(amount, &total_inactive);
        self.fund_transf_module().unstake_swap_transf(unstake_user_id, &swappable)
    }

    #[endpoint(claimPayment)]
    fn claim_payment(&self) -> SCResult<()> {
        let caller = self.get_caller();
        let caller_id = self.user_data().get_user_id(&caller);
        if caller_id == 0 {
            return sc_error!("unknown caller");
        }

        let n_blocks_before_claim =
            self.settings().get_n_blocks_before_force_unstake() +
            self.settings().get_n_blocks_before_unbond();
        let claimed_payments = sc_try!(self.fund_transf_module().claim_all_eligible_deferred_payments(
            caller_id,
            n_blocks_before_claim
        ));

        if claimed_payments > 0 {
            // forward payment to seller
            self.send_tx(&caller, &claimed_payments, "payment for stake");
        }

        Ok(())
    }
}
