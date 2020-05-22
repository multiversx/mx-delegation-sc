
use crate::events::*;
use crate::nodes::*;
use crate::stake_state::*;
use crate::user_data::*;

#[elrond_wasm_derive::module(UserStakeModuleImpl)]
pub trait UserStakeModule {

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(EventsModuleImpl)]
    fn events(&self) -> EventsModuleImpl<T, BigInt, BigUint>;

    #[module(NodeModuleImpl)]
    fn nodes(&self) -> NodeModuleImpl<T, BigInt, BigUint>;

    #[module(UserStakeModuleImpl)]
    fn stake_mod(&self) -> UserStakeModuleImpl<T, BigInt, BigUint>;




    #[view]
    #[storage_get("stake_state")]
    fn stakeState(&self) -> StakeState;

    #[private]
    #[storage_set("stake_state")]
    fn _set_stake_state(&self, active: StakeState);

    /// This is stake that is in the contract, not sent to the auction contract.
    #[private]
    #[storage_get("inactive_stake")]
    fn _get_inactive_stake(&self) -> BigUint;

    #[private]
    #[storage_set("inactive_stake")]
    fn _set_inactive_stake(&self, inactive_stake: &BigUint);

    /// Yields how much stake was added to the contract.
    #[view]
    #[storage_get("filled_stake")]
    fn getFilledStake(&self) -> BigUint;

    #[private]
    #[storage_set("filled_stake")]
    fn _set_filled_stake(&self, filled_stake: &BigUint);

    #[private]
    fn _check_entire_stake_filled(&self) -> Result<(), &'static str> {
        let expected_stake = self.nodes().getExpectedStake();
        if expected_stake == 0 {
            return Err("cannot activate with 0 stake");
        }

        let filled_stake = self.stake_mod().getFilledStake();
        match filled_stake.cmp(&expected_stake) {
            core::cmp::Ordering::Less => {
                Err("cannot activate before all stake has been filled")
            },
            core::cmp::Ordering::Greater => {
                Err("too much stake filled")
            },
            core::cmp::Ordering::Equal => Ok(())
        }
    }

    /// Yields how much a user has staked in the contract.
    #[view]
    fn getStake(&self, user: Address) -> BigUint {
        let user_id = self.user_data().getUserId(&user);
        if user_id == 0 {
            BigUint::zero()
        } else {
            self.user_data()._get_user_stake(user_id)
        }
    }

    /// Staking is possible while the total stake required by the contract has not yet been filled.
    #[payable]
    fn stake(&self, #[payment] payment: BigUint) -> Result<(), &str> {
        if !self.stakeState().is_open() {
            return Err("cannot stake while contract is active"); 
        }

        if payment == 0 {
            return Ok(());
        }

        // keep track of how much of the contract balance is the accumulated stake
        let mut inactive_stake = self._get_inactive_stake();
        inactive_stake += &payment;
        self._set_inactive_stake(&inactive_stake);

        self._process_stake(payment)
    }

    #[private]
    fn _process_stake(&self, payment: BigUint) -> Result<(), &'static str> {
        // increase global filled stake
        let mut filled_stake = self.getFilledStake();
        filled_stake += &payment;
        if &filled_stake > &self.nodes().getExpectedStake() {
            return Err("payment exceeds unfilled total stake");
        }
        self._set_filled_stake(&filled_stake);

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
        let mut user_data = self.user_data()._load_user_data(user_id);
        user_data.personal_stake += &payment;
        self.user_data().store_user_data(user_id, &user_data);

        // log staking event
        self.events().stake_event(&caller, &payment);

        Ok(())
    }

    // UNSTAKE

    fn unstake(&self, amount: BigUint) -> Result<(), &str> {
        if !self.stakeState().is_open() {
            return Err("cannot unstake while contract is active"); 
        }

        if amount == 0 {
            return Ok(());
        }

        let caller = self.get_caller();
        let user_id = self.user_data().getUserId(&caller);
        if user_id == 0 {
            return Err("only delegators can unstake");
        }

        // check stake 
        let mut user_data = self.user_data()._load_user_data(user_id);
        if &amount > &user_data.personal_stake {
            return Err("cannot unstake more than was staked");
        }

        // save decreased stake
        user_data.personal_stake -= &amount;
        self.user_data().store_user_data(user_id, &user_data);

        // keeping track of inactive stake
        let mut inactive_stake = self._get_inactive_stake();
        inactive_stake -= &amount;
        self._set_inactive_stake(&inactive_stake);

        // decrease global filled stake
        let mut filled_stake = self.getFilledStake();
        filled_stake -= &amount;
        self._set_filled_stake(&filled_stake);

        // send stake to delegator
        self.send_tx(&caller, &amount, "delegation unstake");

        // log
        self.events().unstake_event(&caller, &amount);

        Ok(())
    }

}
