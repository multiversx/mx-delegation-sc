
use crate::user_stake_state::*;
use crate::unbond_queue::*;

use crate::events::*;
use crate::node_config::*;
use crate::pause::*;
use crate::settings::*;
use crate::user_data::*;
use crate::node_activation::*;

imports!();

/// Contains endpoints for staking/withdrawing stake.
#[elrond_wasm_derive::module(UserStakeModuleImpl)]
pub trait UserStakeModule {

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(EventsModuleImpl)]
    fn events(&self) -> EventsModuleImpl<T, BigInt, BigUint>;

    #[module(NodeConfigModuleImpl)]
    fn node_config(&self) -> NodeConfigModuleImpl<T, BigInt, BigUint>;

    #[module(NodeActivationModuleImpl)]
    fn node_activation(&self) -> NodeActivationModuleImpl<T, BigInt, BigUint>;

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    #[module(PauseModuleImpl)]
    fn pause(&self) -> PauseModuleImpl<T, BigInt, BigUint>;

    #[payable]
    #[endpoint]
    fn stake(&self, #[payment] payment: BigUint) -> Result<(), SCError> {
        if self.pause().is_staking_paused() {
            return sc_error!("staking paused");
        }
        
        if payment == 0 {
            return Ok(());
        }

        self.process_stake(payment)
    }

    fn process_stake(&self, payment: BigUint) -> Result<(), SCError> {
        // get user id or create user
        // we use user id as an intermediate identifier between user address and data,
        // because we might at some point need to iterate over all user data
        let caller = self.get_caller();
        let mut user_id = self.user_data().get_user_id(&caller);
        if user_id == 0 {
            user_id = self.user_data().new_user();
            self.user_data().set_user_id(&caller, user_id);
        }
        
        // save increased stake
        self.user_data().increase_user_stake_of_type(user_id, UserStakeState::Inactive, &payment);

        // auto-activation, if enabled
        if self.settings().is_auto_activation_enabled() {
            self.node_activation().perform_stake_all_available()?;
        }
        

        // log staking event
        self.events().stake_event(&caller, &payment);

        Ok(())
    }

    // WITHDRAW INACTIVE

    #[endpoint(withdrawInactiveStake)]
    fn withdraw_inactive_stake(&self, amount: BigUint) -> Result<(), SCError> {
        if amount == 0 {
            return Ok(());
        }

        let caller = self.get_caller();
        let user_id = self.user_data().get_user_id(&caller);
        if user_id == 0 {
            return sc_error!("only delegators can withdraw inactive stake");
        }

        // first withdraw from unavailable inactive stake
        let withdraw_stake = self.user_data().get_user_stake_of_type(user_id, UserStakeState::WithdrawOnly);
        if &amount <= &withdraw_stake {
            self.user_data().decrease_user_stake_of_type(user_id, UserStakeState::WithdrawOnly, &amount);
        } else {
            // if that is not enough, retrieve proper inactive stake
            self.user_data().decrease_user_stake_of_type(user_id, UserStakeState::WithdrawOnly, &withdraw_stake);
            let remaining = &amount - &withdraw_stake;
            let enough = self.user_data().decrease_user_stake_of_type(user_id, UserStakeState::Inactive, &remaining);
            if !enough {
                return sc_error!("cannot withdraw more than inactive stake");
            }
        }

        // send stake to delegator
        self.send_tx(&caller, &amount, "delegation withdraw inactive stake");

        // log
        self.events().unstake_event(&caller, &amount);

        Ok(())
    }

    /// Delegators can force some or all nodes to unstake
    /// if they put up stake for sale and no-one has bought it for long enough.
    /// This operation can be performed by any delegator.
    #[endpoint(unStake)]
    fn unstake_endpoint(&self) -> Result<(), SCError> {
        let user_id = self.user_data().get_user_id(&self.get_caller());
        if user_id == 0 {
            return sc_error!("only delegators can call unStake");
        }

        let stake_for_sale = self.user_data().get_user_stake_for_sale(user_id);
        if stake_for_sale == 0 {
            return sc_error!("only delegators that have announced unStake can call unStake");
        }

        let block_nonce_of_stake_offer = self.user_data().get_user_bl_nonce_of_stake_offer(user_id);
        let n_blocks_before_force_unstake = self.settings().get_n_blocks_before_force_unstake();
        if self.get_block_nonce() <= block_nonce_of_stake_offer + n_blocks_before_force_unstake {
            return sc_error!("too soon to call unStake");
        }

        // find nodes to unstake
        let (node_ids, bls_keys) = self.node_config().find_nodes_for_unstake(&stake_for_sale);
        
        let unbond_queue_entry = UnbondQueueItem {
            user_id: user_id,
            amount: stake_for_sale,
        };
        self.node_activation().perform_unstake_nodes(Some(unbond_queue_entry), node_ids, bls_keys)
    }

    #[endpoint(unBond)]
    fn unbond_endpoint(&self) -> Result<(), SCError> {
        let caller = self.get_caller();
        let user_id = self.user_data().get_user_id(&caller);
        if user_id == 0 {
            return sc_error!("only delegators can withdraw inactive stake");
        }

        let mut amount = BigUint::zero();
        let withdraw_stake = self.user_data().get_user_stake_of_type(user_id, UserStakeState::WithdrawOnly);
        if withdraw_stake > 0 {
            // unavailable inactive stake
            amount += &withdraw_stake;
            self.user_data().decrease_user_stake_of_type(user_id, UserStakeState::WithdrawOnly, &withdraw_stake);
        }
        let inactive_stake = self.user_data().get_user_stake_of_type(user_id, UserStakeState::Inactive);
        if inactive_stake > 0 {
            // regular inactive stake
            amount += &inactive_stake;
            self.user_data().decrease_user_stake_of_type(user_id, UserStakeState::Inactive, &inactive_stake);
        }

        // send stake to delegator
        self.send_tx(&caller, &amount, "delegation withdraw inactive stake");

        // log
        self.events().unstake_event(&caller, &amount);

        Ok(())
    }
}
