
use crate::user_stake_state::*;

use crate::events::*;
use crate::node_config::*;
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

    #[payable]
    fn stake(&self, #[payment] payment: BigUint) -> Result<(), &str> {
        if payment == 0 {
            return Ok(());
        }

        self._process_stake(payment)
    }

    #[private]
    fn _process_stake(&self, payment: BigUint) -> Result<(), &'static str> {
        // get user id or create user
        // we use user id as an intermediate identifier between user address and data,
        // because we might at some point need to iterate over all user data
        let caller = self.get_caller();
        let mut user_id = self.user_data().getUserId(&caller);
        if user_id == 0 {
            user_id = self.user_data().new_user();
            self.user_data()._set_user_id(&caller, user_id);
        }
        
        // save increased stake
        self.user_data()._increase_user_stake_of_type(user_id, UserStakeState::Inactive, &payment);

        // log staking event
        self.events().stake_event(&caller, &payment);

        Ok(())
    }

    // UNSTAKE

    fn unStake(&self, amount: BigUint) -> Result<(), &str> {
        if amount == 0 {
            return Ok(());
        }

        let caller = self.get_caller();
        let user_id = self.user_data().getUserId(&caller);
        if user_id == 0 {
            return Err("only delegators can unstake");
        }

        // check that there is enough inactive stake & save decreased stake
        let ok = self.user_data()._decrease_user_stake_of_type(user_id, UserStakeState::Inactive, &amount);
        if !ok {
            return Err("cannot unstake more than was staked");
        }

        // send stake to delegator
        self.send_tx(&caller, &amount, "delegation unstake");

        // log
        self.events().unstake_event(&caller, &amount);

        Ok(())
    }
}
