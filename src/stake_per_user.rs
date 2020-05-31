
use crate::node_state::*;

use crate::events::*;
use crate::nodes::*;
use crate::user_data::*;
use crate::stake_per_node::*;

imports!();

#[elrond_wasm_derive::module(UserStakeModuleImpl)]
pub trait UserStakeModule {

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(EventsModuleImpl)]
    fn events(&self) -> EventsModuleImpl<T, BigInt, BigUint>;

    #[module(NodeModuleImpl)]
    fn nodes(&self) -> NodeModuleImpl<T, BigInt, BigUint>;

    #[module(ContractStakeModuleImpl)]
    fn contract_stake(&self) -> ContractStakeModuleImpl<T, BigInt, BigUint>;

    /// Yields how much a user has staked in the contract.
    #[view]
    fn getUserStake(&self, user_address: Address) -> BigUint {
        let user_id = self.user_data().getUserId(&user_address);
        if user_id == 0 {
            BigUint::zero()
        } else {
            self.user_data()._get_user_total_stake(user_id)
        }
    }

    #[view]
    fn getUserActiveStake(&self, user_address: Address) -> BigUint {
        let user_id = self.user_data().getUserId(&user_address);
        if user_id == 0 {
            BigUint::zero()
        } else {
            self.user_data()._get_user_stake_of_type(user_id, NodeState::Active)
        }
    }

    #[view]
    fn getUserStakeByType(&self, user_address: &Address) -> Vec<BigUint> {
        // TODO: replace result type with something based on tuples
        let user_id = self.user_data().getUserId(&user_address);
        let mut result = Vec::<BigUint>::with_capacity(7);
        if user_id == 0 {
            result.push(BigUint::zero());
            result.push(BigUint::zero());
            result.push(BigUint::zero());
            result.push(BigUint::zero());
            result.push(BigUint::zero());
            result.push(BigUint::zero());
            result.push(BigUint::zero());
        } else {
            result.push(self.user_data()._get_user_stake_of_type(user_id, NodeState::Inactive));
            result.push(self.user_data()._get_user_stake_of_type(user_id, NodeState::PendingActivation));
            result.push(self.user_data()._get_user_stake_of_type(user_id, NodeState::Active));
            result.push(self.user_data()._get_user_stake_of_type(user_id, NodeState::PendingDeactivation));
            result.push(self.user_data()._get_user_stake_of_type(user_id, NodeState::UnBondPeriod));
            result.push(self.user_data()._get_user_stake_of_type(user_id, NodeState::PendingUnBond));
            result.push(self.user_data()._get_user_stake_of_type(user_id, NodeState::Removed));
        }
        
        result
    }

    #[payable]
    fn stake(&self, #[payment] payment: BigUint) -> Result<(), &str> {
        if payment == 0 {
            return Ok(());
        }

        // // keep track of how much of the contract balance is the accumulated stake
        // let mut inactive_stake = self.contract_stake()._get_inactive_stake();
        // inactive_stake += &payment;
        // self.contract_stake()._set_inactive_stake(&inactive_stake);

        self._process_stake(payment)
    }

    #[private]
    fn _process_stake(&self, payment: BigUint) -> Result<(), &'static str> {
        // // increase global filled stake
        // let mut filled_stake = self.contract_stake().getFilledStake();
        // filled_stake += &payment;
        // if &filled_stake > &self.nodes().getExpectedStake() {
        //     return Err("payment exceeds unfilled total stake");
        // }
        // self.contract_stake()._set_filled_stake(&filled_stake);

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
        let mut user_total_stake = self.user_data()._get_user_total_stake(user_id);
        let mut user_inactive_stake = self.user_data()._get_user_stake_of_type(user_id, NodeState::Inactive);
        user_total_stake += &payment;
        user_inactive_stake += &payment;
        self.user_data()._set_user_total_stake(user_id, &user_total_stake);
        self.user_data()._set_user_stake_of_type(user_id, NodeState::Inactive, &user_inactive_stake);

        // log staking event
        self.events().stake_event(&caller, &payment);

        Ok(())
    }

    // UNSTAKE

    fn unstake(&self, amount: BigUint) -> Result<(), &str> {
        if amount == 0 {
            return Ok(());
        }

        let caller = self.get_caller();
        let user_id = self.user_data().getUserId(&caller);
        if user_id == 0 {
            return Err("only delegators can unstake");
        }

        // check that there is enough inactive stake
        let mut user_inactive_stake = self.user_data()._get_user_stake_of_type(user_id, NodeState::Inactive);
        if &amount > &user_inactive_stake {
            return Err("cannot unstake more than was staked");
        }

        // save decreased stake
        let mut user_total_stake = self.user_data()._get_user_total_stake(user_id);
        user_total_stake -= &amount;
        user_inactive_stake -= &amount;
        self.user_data()._set_user_total_stake(user_id, &user_total_stake);
        self.user_data()._set_user_stake_of_type(user_id, NodeState::Inactive, &user_inactive_stake);

        // // keeping track of inactive stake
        // let mut inactive_stake = self.contract_stake()._get_inactive_stake();
        // inactive_stake -= &amount;
        // self.contract_stake()._set_inactive_stake(&inactive_stake);

        // // decrease global filled stake
        // let mut filled_stake = self.contract_stake().getFilledStake();
        // filled_stake -= &amount;
        // self.contract_stake()._set_filled_stake(&filled_stake);

        // send stake to delegator
        self.send_tx(&caller, &amount, "delegation unstake");

        // log
        self.events().unstake_event(&caller, &amount);

        Ok(())
    }


}
