imports!();

// use crate::types::fund_list::*;
// use crate::types::fund_item::*;
use crate::types::fund_type::*;
// use crate::node_config::PERCENTAGE_DENOMINATOR;


use crate::fund_module::*;
use crate::user_data::*;
// use crate::settings::*;

/// Storing total stake per type the same way as we store it for users, but with user_id 0.
/// There can be no user with id 0, so the value is safe to use.
/// These values are redundant. They help avoid having to recompute the sum, especially when computing rewards.
/// At all times the values stored here must be the sums of all user values for the respective stake state,
/// no operation may break this invariant!
pub const USER_STAKE_TOTALS_ID: usize = 0;

#[elrond_wasm_derive::module(FundViewModuleImpl)]
pub trait FundViewModule {

    #[module(FundModuleImpl)]
    fn fund_module(&self) -> FundModuleImpl<T, BigInt, BigUint>;

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    fn get_user_stake_of_type(&self, user_id: usize, fund_type: FundType) -> BigUint {
        if user_id == USER_STAKE_TOTALS_ID {
            let type_list = self.fund_module().get_fund_list_by_type(fund_type);
            type_list.total_balance
        } else {
            let user_list = self.fund_module().get_fund_list_by_user(user_id, fund_type);
            user_list.total_balance
        }
    }

    fn get_user_total_stake(&self, user_id: usize) -> BigUint {
        let mut sum = BigUint::zero();
        sum += self.get_user_stake_of_type(user_id, FundType::WithdrawOnly);
        sum += self.get_user_stake_of_type(user_id, FundType::Waiting);
        sum += self.get_user_stake_of_type(user_id, FundType::PendingActivation);
        sum += self.get_user_stake_of_type(user_id, FundType::ActivationFailed);
        sum += self.get_user_stake_of_type(user_id, FundType::Active);
        sum += self.get_user_stake_of_type(user_id, FundType::UnStaked);
        sum += self.get_user_stake_of_type(user_id, FundType::DeferredPayment);
        sum
    }

    #[view(totalStake)]
    fn get_total_stake(&self) -> BigUint {
        self.get_user_total_stake(USER_STAKE_TOTALS_ID)
    } 

    /// Yields how much a user has staked in the contract.
    #[view(getUserStake)]
    fn get_user_total_stake_endpoint(&self, user_address: Address) -> BigUint {
        let user_id = self.user_data().get_user_id(&user_address);
        if user_id == 0 {
            BigUint::zero()
        } else {
            self.get_user_total_stake(user_id)
        }
    }

    #[view(getUserActiveStake)]
    fn get_user_active_stake_endpoint(&self, user_address: Address) -> BigUint {
        let user_id = self.user_data().get_user_id(&user_address);
        if user_id == 0 {
            BigUint::zero()
        } else {
            self.get_user_stake_of_type(user_id, FundType::Active)
        }
    }

    #[view(getUserInactiveStake)]
    fn get_user_inactive_stake_endpoint(&self, user_address: Address) -> BigUint {
        let user_id = self.user_data().get_user_id(&user_address);
        if user_id == 0 {
            BigUint::zero()
        } else {
            self.get_user_stake_of_type(user_id, FundType::Waiting) +
            self.get_user_stake_of_type(user_id, FundType::WithdrawOnly)
        }
    }

    fn get_user_stake_by_type(&self, user_id: usize) -> MultiResult7<BigUint, BigUint, BigUint, BigUint, BigUint, BigUint, BigUint> {
        (
            self.get_user_stake_of_type(user_id, FundType::WithdrawOnly),
            self.get_user_stake_of_type(user_id, FundType::Waiting),
            self.get_user_stake_of_type(user_id, FundType::PendingActivation),
            self.get_user_stake_of_type(user_id, FundType::Active),
            self.get_user_stake_of_type(user_id, FundType::ActivationFailed),
            self.get_user_stake_of_type(user_id, FundType::UnStaked),
            self.get_user_stake_of_type(user_id, FundType::DeferredPayment),
        ).into()
    }

    #[view(getUserStakeByType)]
    fn get_user_stake_by_type_endpoint(&self, user_address: &Address) -> MultiResult7<BigUint, BigUint, BigUint, BigUint, BigUint, BigUint, BigUint> {
        let user_id = self.user_data().get_user_id(&user_address);
        if user_id == 0 {
            (
                BigUint::zero(),
                BigUint::zero(),
                BigUint::zero(),
                BigUint::zero(),
                BigUint::zero(),
                BigUint::zero(),
                BigUint::zero(),
            ).into()
        } else {
            self.get_user_stake_by_type(user_id)
        }
    }

    #[view(getTotalStakeByType)]
    fn get_total_stake_by_type_endpoint(&self) -> MultiResult7<BigUint, BigUint, BigUint, BigUint, BigUint, BigUint, BigUint> {
        self.get_user_stake_by_type(USER_STAKE_TOTALS_ID)
    }

    #[view(getTotalActiveStake)]
    fn get_total_active_stake(&self) -> BigUint {
        self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active)
    }

    #[view(getTotalInactiveStake)]
    fn get_total_inactive_stake(&self) -> BigUint {
        self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Waiting) +
        self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::WithdrawOnly)
    }

}
