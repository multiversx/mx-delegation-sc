imports!();

use crate::settings::*;
use crate::fund_module::*;
use crate::fund_type::*;

use crate::node_config::PERCENTAGE_DENOMINATOR;
use crate::user_stake_state::*;
// use crate::unbond_queue::*;

// Groups together data per delegator from the storage.
pub struct UserRewardData<BigUint> {
    /// The value of the total cumulated rewards in the contract when the user's rewards were computed the last time.
    pub reward_checkpoint: BigUint,

    /// Rewards that are computed but not yet sent to the delegator.
    pub unclaimed_rewards: BigUint,
}

/// Storing total stake per type the same way as we store it for users, but with user_id 0.
/// There can be no user with id 0, so the value is safe to use.
/// These values are redundant. They help avoid having to recompute the sum, especially when computing rewards.
/// At all times the values stored here must be the sums of all user values for the respective stake state,
/// no operation may break this invariant!
pub const USER_STAKE_TOTALS_ID: usize = 0;

/// Deals with storage data about delegators.
#[elrond_wasm_derive::module(UserDataModuleImpl)]
pub trait UserDataModule {

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    #[module(FundModuleImpl)]
    fn fund_module(&self) -> FundModuleImpl<T, BigInt, BigUint>;

    /// Each delegator gets a user id. This is in order to be able to iterate over their data.
    /// This is a mapping from delegator address to delegator id.
    /// The key is the bytes "user_id" concatenated with their public key.
    /// The value is the user id.
    #[view(getUserId)]
    #[storage_get("user_id")]
    fn get_user_id(&self, address: &Address) -> usize;

    #[storage_set("user_id")]
    fn set_user_id(&self, address: &Address, user_id: usize);

    /// Nr delegators + 1 (the node address)
    #[view(getNumUsers)]
    #[storage_get("num_users")]
    fn get_num_users(&self) -> usize;

    /// Yields how accounts are registered in the contract.
    /// Note that not all of them must have stakes greater than zero.
    #[storage_set("num_users")]
    fn set_num_users(&self, num_users: usize);

    // creates new user id
    fn new_user(&self) -> usize {
        let mut num_users = self.get_num_users();
        num_users += 1;
        self.set_num_users(num_users);
        num_users
    }

    #[view(totalStake)]
    fn get_total_stake(&self) -> BigUint {
        self.fund_module().query_all(|fund_info| fund_info.fund_type.is_stake())
    } 

    // #[view(totalStake)]
    // #[storage_get("total_stake")]
    // fn get_total_stake(&self) -> BigUint;

    // #[storage_get_mut("total_stake")]
    // fn get_mut_total_stake(&self) -> mut_storage!(BigUint);

    // /// How much a delegator has staked.
    // #[storage_get("u_stake_totl")]
    // fn get_user_total_stake(&self, user_id: usize) -> BigUint;

    // #[storage_get_mut("u_stake_totl")]
    // fn get_mut_user_total_stake(&self, user_id: usize) -> mut_storage!(BigUint);

    // /// How much of a delegator's has been sent to the auction SC.
    // #[storage_get("u_stake_type")]
    // fn get_user_stake_of_type(&self, user_id: usize, stake_type: UserStakeState) -> BigUint;

    fn get_user_stake_of_type(&self, user_id: usize, stake_type: UserStakeState) -> BigUint {
        match stake_type {
            UserStakeState::Inactive => {
                self.fund_module().query_list(DISCR_FREE,
                    |fund_info|
                        (user_id == USER_STAKE_TOTALS_ID || fund_info.user_id == user_id) &&
                        if let FundType::Free{ requested_unstake: false } = fund_info.fund_type { true } else { false }
                )
            },
            UserStakeState::PendingActivation => {
                self.fund_module().query_list(DISCR_PENDING_ACT,
                    |fund_info| 
                        (user_id == USER_STAKE_TOTALS_ID || fund_info.user_id == user_id))
            },
            UserStakeState::Active => {
                self.fund_module().query_list(DISCR_ACTIVE, 
                    |fund_info|
                        (user_id == USER_STAKE_TOTALS_ID || fund_info.user_id == user_id))
            },
            UserStakeState::PendingDeactivation => {
                self.fund_module().query_list(DISCR_PENDING_DEACT,
                    |fund_info|
                        (user_id == USER_STAKE_TOTALS_ID || fund_info.user_id == user_id) &&
                        if let FundType::PendingDeactivation{ requested_unstake: false } = fund_info.fund_type { true } else { false }
                )
            },
            UserStakeState::UnBondPeriod => {
                self.fund_module().query_list(DISCR_UNBOND, 
                    |fund_info|
                        (user_id == USER_STAKE_TOTALS_ID || fund_info.user_id == user_id))
            },
            UserStakeState::PendingUnBond => {
                self.fund_module().query_list(DISCR_PENDING_ACT, 
                    |fund_info|
                        (user_id == USER_STAKE_TOTALS_ID || fund_info.user_id == user_id))
            },
            UserStakeState::WithdrawOnly => {
                self.fund_module().query_list(DISCR_FREE,
                    |fund_info|
                        (user_id == USER_STAKE_TOTALS_ID || fund_info.user_id == user_id) &&
                        if let FundType::Free{ requested_unstake: true } = fund_info.fund_type { true } else { false })
            },
            UserStakeState::ActivationFailed => {
                self.fund_module().query_list(DISCR_ACTIVE_FAILED, 
                    |fund_info|
                        (user_id == USER_STAKE_TOTALS_ID || fund_info.user_id == user_id))
            },
            UserStakeState::ActiveForSale => {
                self.fund_module().query_list(DISCR_ACTIVE_FOR_SALE, 
                    |fund_info|
                        (user_id == USER_STAKE_TOTALS_ID || fund_info.user_id == user_id))
            },
            UserStakeState::PendingDeactivationFromSale => {
                self.fund_module().query_list(DISCR_PENDING_DEACT,
                    |fund_info|
                        (user_id == USER_STAKE_TOTALS_ID || fund_info.user_id == user_id) &&
                        if let FundType::PendingDeactivation{ requested_unstake: true } = fund_info.fund_type { true } else { false }
                )
            },
        }
    }

    // #[storage_get_mut("u_stake_type")]
    // fn get_mut_user_stake_of_type(&self, user_id: usize, stake_type: UserStakeState) -> mut_storage!(BigUint);

    // fn increase_user_stake_of_type(&self, user_id: usize, stake_type: UserStakeState, amount: &BigUint) {
    //     *self.get_mut_user_stake_of_type(user_id, stake_type) += amount;
    //     *self.get_mut_user_stake_of_type(USER_STAKE_TOTALS_ID, stake_type) += amount;
    //     *self.get_mut_user_total_stake(user_id) += amount;
    //     *self.get_mut_total_stake() += amount;
    // }

    // fn decrease_user_stake_of_type(&self, user_id: usize, stake_type: UserStakeState, amount: &BigUint) -> bool {
    //     let mut user_st_value = self.get_mut_user_stake_of_type(user_id, stake_type);
    //     if amount > &*user_st_value {
    //         return false;
    //     }
    //     *user_st_value -= amount;
    //     *self.get_mut_user_stake_of_type(USER_STAKE_TOTALS_ID, stake_type) -= amount;
    //     *self.get_mut_user_total_stake(user_id) -= amount;
    //     *self.get_mut_total_stake() -= amount;
    //     true
    // }


    fn get_user_total_stake(&self, user_id: usize) -> BigUint {
        self.fund_module().query_all(
            |fund_info| fund_info.fund_type.is_stake() && fund_info.user_id == user_id)
    }

    /// Yields how much a user has staked in the contract.
    #[view(getUserStake)]
    fn get_user_total_stake_endpoint(&self, user_address: Address) -> BigUint {
        let user_id = self.get_user_id(&user_address);
        if user_id == 0 {
            BigUint::zero()
        } else {
            self.get_user_total_stake(user_id)
        }
    }

    #[view(getUserActiveStake)]
    fn get_user_active_stake_endpoint(&self, user_address: Address) -> BigUint {
        let user_id = self.get_user_id(&user_address);
        if user_id == 0 {
            BigUint::zero()
        } else {
            self.get_user_stake_of_type(user_id, UserStakeState::Active)
        }
    }

    #[view(getUserInactiveStake)]
    fn get_user_inactive_stake_endpoint(&self, user_address: Address) -> BigUint {
        let user_id = self.get_user_id(&user_address);
        if user_id == 0 {
            BigUint::zero()
        } else {
            self.get_user_stake_of_type(user_id, UserStakeState::Inactive) +
            self.get_user_stake_of_type(user_id, UserStakeState::WithdrawOnly)
        }
    }

    fn get_user_stake_by_type(&self, user_id: usize) -> MultiResult10<BigUint, BigUint, BigUint, BigUint, BigUint, BigUint, BigUint, BigUint, BigUint, BigUint> {
        (
            self.get_user_stake_of_type(user_id, UserStakeState::Inactive),
            self.get_user_stake_of_type(user_id, UserStakeState::PendingActivation),
            self.get_user_stake_of_type(user_id, UserStakeState::Active),
            self.get_user_stake_of_type(user_id, UserStakeState::PendingDeactivation),
            self.get_user_stake_of_type(user_id, UserStakeState::UnBondPeriod),
            self.get_user_stake_of_type(user_id, UserStakeState::PendingUnBond),
            self.get_user_stake_of_type(user_id, UserStakeState::WithdrawOnly),
            self.get_user_stake_of_type(user_id, UserStakeState::ActivationFailed),
            self.get_user_stake_of_type(user_id, UserStakeState::ActiveForSale),
            self.get_user_stake_of_type(user_id, UserStakeState::PendingDeactivationFromSale),
        ).into()
    }

    #[view(getUserStakeByType)]
    fn get_user_stake_by_type_endpoint(&self, user_address: &Address) -> MultiResult10<BigUint, BigUint, BigUint, BigUint, BigUint, BigUint, BigUint, BigUint, BigUint, BigUint> {
        let user_id = self.get_user_id(&user_address);
        if user_id == 0 {
            (
                BigUint::zero(),
                BigUint::zero(),
                BigUint::zero(),
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
    fn get_total_stake_by_type_endpoint(&self) -> MultiResult10<BigUint, BigUint, BigUint, BigUint, BigUint, BigUint, BigUint, BigUint, BigUint, BigUint> {
        self.get_user_stake_by_type(USER_STAKE_TOTALS_ID)
    }

    #[view(getTotalActiveStake)]
    fn get_total_active_stake(&self) -> BigUint {
        self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, UserStakeState::Active)
    }

    #[view(getTotalInactiveStake)]
    fn get_total_inactive_stake(&self) -> BigUint {
        self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, UserStakeState::Inactive) +
        self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, UserStakeState::WithdrawOnly)
    }

    fn all_funds_in_contract(&self) -> BigUint {
        self.fund_module().query_all(|fund_info| fund_info.fund_type.funds_in_contract())
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
        let user_id = self.get_user_id(&user);
        if user_id == 0 {
            return MultiResultVec::new();
        }
        self.fund_module().get_fund_list(DISCR_ACTIVE_FOR_SALE)
            .0.iter()
            .filter_map(|fund_item| {
                if fund_item.info.user_id == user_id {
                    if let FundType::ActiveForSale{ created } = fund_item.info.fund_type {
                        return Some(created)
                    }
                }
                None
            })
            .collect()
    }

    /// Claiming rewards has 2 steps:
    /// 1. computing the delegator rewards out of the total rewards, and
    /// 2. sending those rewards to the delegator address. 
    /// This field keeps track of rewards that went through step 1 but not 2,
    /// i.e. were computed and deducted from the total rewards, but not yet "physically" sent to the user.
    /// The unclaimed stake still resides in the contract.
    #[storage_get("u_rew_unclmd")]
    fn get_user_rew_unclaimed(&self, user_id: usize) -> BigUint;

    #[storage_set("u_rew_unclmd")]
    fn set_user_rew_unclaimed(&self, user_id: usize, user_rew_unclaimed: &BigUint);

    /// As the time passes, if the contract is active, rewards periodically arrive in the contract. 
    /// Users can claim their share of rewards anytime.
    /// This field helps keeping track of how many rewards came to the contract since the last claim.
    /// More specifically, it indicates the cumulated sum of rewards that had arrived in the contract 
    /// when the user last claimed their own personal rewards.
    /// If zero, it means the user never claimed any rewards.
    /// If equal to get_total_cumulated_rewards, it means the user claimed everything there is for him/her.
    #[storage_get("u_rew_checkp")]
    fn get_user_rew_checkpoint(&self, user_id: usize) -> BigUint;

    #[storage_set("u_rew_checkp")]
    fn set_user_rew_checkpoint(&self, user_id: usize, user_rew_checkpoint: &BigUint);

    // /// Users can trade stake. To do so, a user must first offer stake for sale.
    // /// This field keeps track of how much stake each user has offered for sale.
    // #[storage_get("u_stake_sale")]
    // fn get_user_stake_for_sale(&self, user_id: usize) -> BigUint;

    // #[storage_set("u_stake_sale")]
    // fn set_user_stake_for_sale(&self, user_id: usize, user_stake_for_sale: &BigUint);

    // // TODO: auto-generate
    // fn update_user_stake_for_sale<F, R>(&self, user_id: usize, f: F) -> R
    // where F: Fn(&mut BigUint) -> R
    // {
    //     let mut value = self.get_user_stake_for_sale(user_id);
    //     let result = f(&mut value);
    //     self.set_user_stake_for_sale(user_id, &value);
    //     result
    // }

    /// Loads the entire UserRewardData object from storage.
    fn load_user_data(&self, user_id: usize) -> UserRewardData<BigUint> {
        let u_rew_checkp = self.get_user_rew_checkpoint(user_id);
        let u_rew_unclmd = self.get_user_rew_unclaimed(user_id);
        UserRewardData {
            reward_checkpoint: u_rew_checkp,
            unclaimed_rewards: u_rew_unclmd,
        }
    }

    /// Saves a UserRewardData object to storage.
    fn store_user_data(&self, user_id: usize, data: &UserRewardData<BigUint>) {
        self.set_user_rew_checkpoint(user_id, &data.reward_checkpoint);
        self.set_user_rew_unclaimed(user_id, &data.unclaimed_rewards);
    }

    // /// Block timestamp of when the user offered stake for sale.
    // /// Note: not part of the UserRewardData struct because it is not needed as often.
    // #[storage_get("u_stake_toff")]
    // fn get_user_bl_nonce_of_stake_offer(&self, user_id: usize) -> u64;

    // #[storage_set("u_stake_toff")]
    // fn set_user_bl_nonce_of_stake_offer(&self, user_id: usize, bl_nonce_of_stake_offer: u64);

    // fn convert_user_stake(&self, user_id: usize, old_type: UserStakeState, new_type: UserStakeState, total_supply: &mut BigUint) {
    //     let mut user_stake_old_type = self.get_mut_user_stake_of_type(user_id, old_type);
    //     if *total_supply > *user_stake_old_type {
    //         *self.get_mut_user_stake_of_type(user_id, new_type) += &*user_stake_old_type;
    //         *self.get_mut_user_stake_of_type(USER_STAKE_TOTALS_ID, new_type) += &*user_stake_old_type;
    //         *self.get_mut_user_stake_of_type(USER_STAKE_TOTALS_ID, old_type) -= &*user_stake_old_type;
    //         *total_supply -= &*user_stake_old_type;
    //         *user_stake_old_type = BigUint::zero();
    //     } else {
    //         *user_stake_old_type -= &*total_supply;
    //         *self.get_mut_user_stake_of_type(user_id, new_type) += &*total_supply;
    //         *self.get_mut_user_stake_of_type(USER_STAKE_TOTALS_ID, new_type) += &*total_supply;
    //         *self.get_mut_user_stake_of_type(USER_STAKE_TOTALS_ID, old_type) -= &*total_supply;
    //         *total_supply = BigUint::zero();
    //     }
    // }

    // /// Converts from one type of stake to another, for as many users as possible,
    // /// within the limit of total_supply.
    // /// Argument total_supply decreases in the process.
    // /// Walking in increasing user id order, so older users get picked first.
    // fn convert_user_stake_asc(&self, old_type: UserStakeState, new_type: UserStakeState, total_supply: &mut BigUint) {
    //     let num_users = self.get_num_users();
    //     let mut i = 1usize;
    //     while i <= num_users && *total_supply > 0 {
    //         self.convert_user_stake(i, old_type, new_type, total_supply);
    //         i += 1;
    //     }
    // }

    // /// Converts from one type of stake to another, for as many users as possible,
    // /// within the limit of total_supply.
    // /// Argument total_supply decreases in the process.
    // /// Walking in decreasing user id order, so newer users get picked first.
    // fn convert_user_stake_desc(&self, old_type: UserStakeState, new_type: UserStakeState, total_supply: &mut BigUint) {
    //     let mut i = self.get_num_users();
    //     while i > 0 && *total_supply > 0 {
    //         self.convert_user_stake(i, old_type, new_type, total_supply);
    //         i -= 1;
    //     }
    // }

    // #[view(getUnbondQueue)]
    // #[storage_get("unbond_queue")]
    // fn get_unbond_queue(&self) -> Queue<UnbondQueueItem<BigUint>>;

    // #[storage_set("unbond_queue")]
    // fn set_unbond_queue(&self, unbond_queue: Queue<UnbondQueueItem<BigUint>>);

    
}
