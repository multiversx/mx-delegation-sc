use super::node_storage::node_config::*;
use super::user_fund_storage::fund_transf_module::*;
use super::user_fund_storage::user_data::*;
use crate::reset_checkpoint_types::*;
use crate::reset_checkpoints::*;
use crate::rewards::*;

use core::num::NonZeroUsize;

elrond_wasm::imports!();

/// Indicates how we express the percentage of rewards that go to the node.
/// Since we cannot have floating point numbers, we use fixed point with this denominator.
/// Percents + 2 decimals -> 10000.
pub static PERCENTAGE_DENOMINATOR: usize = 10000;

/// Validator reward destination will always be user with id 1.
/// This can also count as a delegator (if the owner adds stake into the contract) or not.
pub static OWNER_USER_ID: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1) };

/// The module deals with initializaton and the global contract settings.
///
#[elrond_wasm_derive::module(SettingsModuleImpl)]
pub trait SettingsModule {
    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(FundTransformationsModuleImpl)]
    fn fund_transf_module(&self) -> FundTransformationsModuleImpl<T, BigInt, BigUint>;

    #[module(NodeConfigModuleImpl)]
    fn node_config(&self) -> NodeConfigModuleImpl<T, BigInt, BigUint>;

    #[module(RewardsModuleImpl)]
    fn rewards(&self) -> RewardsModuleImpl<T, BigInt, BigUint>;

    #[module(ResetCheckpointsModuleImpl)]
    fn reset_checkpoints(&self) -> ResetCheckpointsModuleImpl<T, BigInt, BigUint>;

    /// Yields the address of the contract with which staking will be performed.
    /// This address is standard in the protocol, but it is saved in storage to avoid hardcoding it.
    #[view(getAuctionContractAddress)]
    #[storage_get("auction_addr")]
    fn get_auction_contract_address(&self) -> Address;

    #[storage_set("auction_addr")]
    fn set_auction_addr(&self, auction_addr: &Address);

    /// The proportion of rewards that goes to the owner as compensation for running the nodes.
    /// 10000 = 100%.
    #[view(getServiceFee)]
    #[storage_get("service_fee")]
    fn get_service_fee(&self) -> BigUint;

    #[storage_set("service_fee")]
    fn set_service_fee(&self, service_fee: BigUint);

    /// The stake per node can be changed by the owner.
    /// It does not get set in the contructor, so the owner has to manually set it after the contract is deployed.
    #[endpoint(setServiceFee)]
    fn set_service_fee_endpoint(
        &self,
        service_fee_per_10000: usize,
    ) -> SCResult<OperationCompletionStatus> {
        only_owner!(self, "only owner can change service fee");

        require!(
            service_fee_per_10000 <= PERCENTAGE_DENOMINATOR,
            "service fee out of range"
        );

        require!(
            !self.reset_checkpoints().is_global_op_in_progress(),
            "global checkpoint is in progress"
        );

        let new_service_fee = BigUint::from(service_fee_per_10000);
        if self.get_service_fee() == new_service_fee {
            return Ok(OperationCompletionStatus::Completed);
        }

        if self.is_bootstrap_mode() {
            // no rewards to compute
            // change service fee directly
            self.set_service_fee(new_service_fee);
            Ok(OperationCompletionStatus::Completed)
        } else {
            // start compute all rewards
            self.reset_checkpoints().continue_global_operation(Box::new(
                GlobalOpCheckpoint::ChangeServiceFee {
                    new_service_fee,
                    compute_rewards_data: ComputeAllRewardsData::new(
                        self.rewards().get_total_cumulated_rewards(),
                    ),
                },
            ))
        }
    }

    #[view(getTotalDelegationCap)]
    #[storage_get("total_delegation_cap")]
    fn get_total_delegation_cap(&self) -> BigUint;

    #[storage_set("total_delegation_cap")]
    fn set_total_delegation_cap(&self, amount: BigUint);

    #[view(isBootstrapMode)]
    #[storage_get("bootstrap_mode")]
    fn is_bootstrap_mode(&self) -> bool;

    #[storage_set("bootstrap_mode")]
    fn set_bootstrap_mode(&self, bootstrap_mode: bool);

    /// The minimum proportion of stake that has to be provided by the owner.
    /// 10000 = 100%.
    #[view(getOwnerMinStakeShare)]
    #[storage_get("owner_min_stake_share")]
    fn get_owner_min_stake_share(&self) -> BigUint;

    #[storage_set("owner_min_stake_share")]
    fn set_owner_min_stake_share(&self, owner_min_stake_share: usize);

    fn set_owner_min_stake_share_validated(
        &self,
        owner_min_stake_share_per_10000: usize,
    ) -> SCResult<()> {
        require!(
            owner_min_stake_share_per_10000 <= PERCENTAGE_DENOMINATOR,
            "owner min stake share out of range"
        );

        self.set_owner_min_stake_share(owner_min_stake_share_per_10000);
        Ok(())
    }

    /// Minimum number of n_blocks between unstake and fund getting into inactive state.
    #[view(getNumBlocksBeforeUnBond)]
    #[storage_get("n_blocks_before_unbond")]
    fn get_n_blocks_before_unbond(&self) -> u64;

    #[storage_set("n_blocks_before_unbond")]
    fn set_n_blocks_before_unbond(&self, n_blocks_before_unbond: u64);

    #[endpoint(setNumBlocksBeforeUnBond)]
    fn set_n_blocks_before_unbond_endpoint(&self, n_blocks_before_unbond: u64) -> SCResult<()> {
        only_owner!(self, "only owner can set num blocks before unbond");
        self.set_n_blocks_before_unbond(n_blocks_before_unbond);
        Ok(())
    }

    /// Delegators are not allowed make transactions with less then this amount of stake (of any type).
    /// Zero means disabled.
    #[view(getMinimumStake)]
    #[storage_get("min_stake")]
    fn get_minimum_stake(&self) -> BigUint;

    #[storage_set("min_stake")]
    fn set_minimum_stake(&self, minimum_stake: &BigUint);

    #[endpoint(setMinimumStake)]
    fn set_minimum_stake_endpoint(&self, minimum_stake: BigUint) -> SCResult<()> {
        only_owner!(self, "only owner can set minimum stake");
        self.set_minimum_stake(&minimum_stake);
        Ok(())
    }
}
