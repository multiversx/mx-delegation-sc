
use crate::events::*;
use crate::rewards::*;
use crate::settings::*;
use crate::reset_checkpoints::*;
use crate::user_stake::*;
use user_fund_storage::user_data::*;
use user_fund_storage::fund_transf_module::*;
use user_fund_storage::fund_view_module::*;
use user_fund_storage::types::*;
use elrond_wasm_module_pause::*;

use core::num::NonZeroUsize;
use core::cmp::Ordering;

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

    fn validate_unstake_amount(&self, user_id: usize, amount: &BigUint) -> SCResult<()> {
        let max_unstake = 
            self.fund_view_module().get_user_stake_of_type(user_id, FundType::Waiting) +
            self.fund_view_module().get_user_stake_of_type(user_id, FundType::Active);
        match amount.cmp(&max_unstake) {
            Ordering::Greater =>
                sc_error!("cannot unstake more than the user waiting + active stake"),
            Ordering::Equal => Ok(()),
            Ordering::Less => {
                require!(*amount >= self.settings().get_minimum_stake(),
                    "cannot unstake less than minimum stake");
                Ok(())
            }
        }
    }

    /// unStake - the user will announce that he wants to get out of the contract
    /// selected funds will change from active to inactive, but claimable only after unBond period ends
    #[endpoint(unStake)]
    fn unstake_endpoint(&self, amount: BigUint) -> SCResult<()> {
        require!(self.pause().not_paused(), "contract paused");

        require!(!self.reset_checkpoints().is_global_op_in_progress(),
            "unstaking is temporarily paused as checkpoint is reset");
        
        let caller = self.get_caller();
        let unstake_user_id = non_zero_usize!(
            self.user_data().get_user_id(&caller),
            "only delegators can unstake");

        // validate that amount does not exceed existing waiting + active stake
        sc_try!(self.validate_unstake_amount(unstake_user_id.get(), &amount));

        // first try to remove funds from waiting list
        let mut remaining = amount;
        self.fund_transf_module().swap_user_waiting_to_withdraw_only(unstake_user_id.get(), &mut remaining);
        if remaining == 0 {
            // waiting list entries covered the whole sum
            return Ok(());
        }

        // compute rewards before converting Active -> UnStaked
        self.rewards().compute_one_user_reward(OWNER_USER_ID);
        self.rewards().compute_one_user_reward(unstake_user_id);

        // convert Active -> UnStaked
        self.fund_transf_module().swap_user_active_to_unstaked(unstake_user_id.get(), &mut remaining);
        require!(remaining == 0, "error converting Active to UnStaked");

        // move funds around
        sc_try!(self.user_stake().use_waiting_to_replace_unstaked());

        // check that minimum stake was not violated
        sc_try!(self.user_stake().validate_user_minimum_stake(unstake_user_id.get()));

        Ok(())
    }

    #[view(getUnStakeable)]
    fn get_unstakeable(&self, user_address: Address) -> BigUint {
        let user_id = self.user_data().get_user_id(&user_address);
        if user_id == 0 {
            BigUint::zero()
        } else {
            self.fund_view_module().get_user_stake_of_type(user_id, FundType::Waiting) +
            self.fund_view_module().get_user_stake_of_type(user_id, FundType::Active)
        }
    }

    #[endpoint(unBond)]
    fn unbond_user(&self) -> SCResult<()> {
        require!(self.pause().not_paused(), "contract paused");

        let caller = self.get_caller();
        let caller_id = self.user_data().get_user_id(&caller);
        require!(caller_id > 0, "unknown caller");

        let n_blocks_before_unbond = self.settings().get_n_blocks_before_unbond();
        let _ = self.fund_transf_module().swap_eligible_deferred_to_withdraw(
            caller_id,
            n_blocks_before_unbond
        );

        let amount_liquidated = self.fund_transf_module().liquidate_all_withdraw_only(caller_id);

        if amount_liquidated > 0 {
            // forward payment to seller
            self.send_tx(&caller, &amount_liquidated, "delegation stake unbond");
        }

        Ok(())
    }

    #[view(getUnBondable)]
    fn get_unbondable(&self, user_address: Address) -> BigUint {
        let user_id = self.user_data().get_user_id(&user_address);
        if user_id == 0 {
            BigUint::zero()
        } else {
            let n_blocks_before_unbond = self.settings().get_n_blocks_before_unbond();
            self.fund_view_module().eligible_deferred_payment(user_id, n_blocks_before_unbond) +
            self.fund_view_module().get_user_stake_of_type(user_id, FundType::Active)
        }
    }
}
