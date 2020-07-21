imports!();

// use super::fund_list::*;
// use super::fund_item::*;
use super::fund_type::*;
use crate::node_config::PERCENTAGE_DENOMINATOR;


use crate::fund_module::*;
use crate::user_data::*;
use crate::settings::*;

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

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    #[view(totalStake)]
    fn get_total_stake(&self) -> BigUint {
        self.fund_module().query_all(|fund_info| fund_info.fund_desc.is_stake())
    } 

    fn get_user_stake_of_type(&self, user_id: usize, stake_type: FundType) -> BigUint {
        match stake_type {
            FundType::WithdrawOnly => {
                self.fund_module().query_list(DISCR_WITHDRAW_ONLY,
                    |fund_info|
                        (user_id == USER_STAKE_TOTALS_ID || fund_info.user_id == user_id)
                )
            },
            FundType::Inactive => {
                self.fund_module().query_list(DISCR_INACTIVE,
                    |fund_info|
                        (user_id == USER_STAKE_TOTALS_ID || fund_info.user_id == user_id)
                )
            },
            FundType::PendingActivation => {
                self.fund_module().query_list(DISCR_PENDING_ACT,
                    |fund_info| 
                        (user_id == USER_STAKE_TOTALS_ID || fund_info.user_id == user_id))
            },
            FundType::Active => {
                self.fund_module().query_list(DISCR_ACTIVE, 
                    |fund_info|
                        (user_id == USER_STAKE_TOTALS_ID || fund_info.user_id == user_id))
            },
            FundType::ActivationFailed => {
                self.fund_module().query_list(DISCR_ACTIVE_FAILED, 
                    |fund_info|
                        (user_id == USER_STAKE_TOTALS_ID || fund_info.user_id == user_id))
            },
            FundType::UnStaked => {
                self.fund_module().query_list(DISCR_UNSTAKED, 
                    |fund_info|
                        (user_id == USER_STAKE_TOTALS_ID || fund_info.user_id == user_id))
            },
            FundType::DeferredPayment => {
                self.fund_module().query_list(DISCR_DEF_PAYMENT, 
                    |fund_info|
                        (user_id == USER_STAKE_TOTALS_ID || fund_info.user_id == user_id))
            },
        }
    }


    fn get_user_total_stake(&self, user_id: usize) -> BigUint {
        self.fund_module().query_all(
            |fund_info| fund_info.fund_desc.is_stake() && fund_info.user_id == user_id)
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
            self.get_user_stake_of_type(user_id, FundType::Inactive) +
            self.get_user_stake_of_type(user_id, FundType::WithdrawOnly)
        }
    }

    fn get_user_stake_by_type(&self, user_id: usize) -> MultiResult7<BigUint, BigUint, BigUint, BigUint, BigUint, BigUint, BigUint> {
        (
            self.get_user_stake_of_type(user_id, FundType::WithdrawOnly),
            self.get_user_stake_of_type(user_id, FundType::Inactive),
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
        self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Inactive) +
        self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::WithdrawOnly)
    }

    fn all_funds_in_contract(&self) -> BigUint {
        self.fund_module().query_all(|fund_info| fund_info.fund_desc.funds_in_contract())
    }

    fn validate_total_user_stake(&self, user_id: usize) -> SCResult<()> {
        let user_total = self.get_user_total_stake(user_id);
        if user_total > 0 && user_total < self.settings().get_minimum_stake() {
            return sc_error!("cannot have less stake than minimum stake");
        }
        Ok(())
    }

    fn validate_owner_stake_share(&self) -> SCResult<()> {
        // owner total stake / contract total stake < owner_min_stake_share / 10000
        // reordered to avoid divisions
        if self.get_user_total_stake(OWNER_USER_ID) * BigUint::from(PERCENTAGE_DENOMINATOR) <
            self.get_total_stake() * self.settings().get_owner_min_stake_share() {
                return sc_error!("owner doesn't have enough stake in the contract");
            }
        Ok(())
    }

    #[view(getStakeForSaleCreationNonces)]
    fn get_stake_for_sale_creation_nonces(&self, user: Address) -> MultiResultVec<u64> {
        let user_id = self.user_data().get_user_id(&user);
        if user_id == 0 {
            return MultiResultVec::new();
        }
        self.fund_module().get_fund_list(DISCR_UNSTAKED)
            .0.iter()
            .filter_map(|fund_item| {
                if fund_item.info.user_id == user_id {
                    if let FundDescription::UnStaked{ created } = fund_item.info.fund_desc {
                        return Some(created)
                    }
                }
                None
            })
            .collect()
    }
}
