use crate::settings::OWNER_USER_ID;
use core::num::NonZeroUsize;
use user_fund_storage::types::{FundDescription, FundType};

elrond_wasm::imports!();

pub const DUST_GASLIMIT: u64 = 20_000_000;

/// Functionality for cleaning up very small amounts left in the waiting list.
#[elrond_wasm::derive::module]
pub trait UserStakeDustCleanupModule:
    crate::user_stake_state::UserStakeStateModule
    + crate::reset_checkpoint_state::ResetCheckpointStateModule
    + crate::rewards_state::RewardStateModule
    + crate::settings::SettingsModule
    + crate::events::EventsModule
    + user_fund_storage::user_data::UserDataModule
    + user_fund_storage::fund_module::FundModule
    + user_fund_storage::fund_view_module::FundViewModule
    + user_fund_storage::fund_transf_module::FundTransformationsModule
{
    /// Raw id of the last checkpoint reached by any of the dust cleanup endpoints.
    #[view(dustCleanupCheckpoint)]
    #[storage_mapper("dust_cleanup_checkpoint")]
    fn dust_cleanup_checkpoint(&self) -> SingleValueMapper<usize>;

    /// Counts fund buckets in the waiting list that are below a certain threshold.
    /// Unlike most views, yields the number of entries, rather than the sum of EGLD.
    #[view(countDustItemsWaitingList)]
    fn count_dust_items_waiting_list(&self, dust_limit: &BigUint) -> usize {
        self.count_fund_items_by_type(FundType::Waiting, |fund_item| {
            &fund_item.balance < dust_limit
        })
    }

    /// Counts fund buckets in the active staking list that are below a certain threshold.
    /// Unlike most views, yields the number of entries, rather than the sum of EGLD.
    #[view(countDustItemsActive)]
    fn count_dust_items_active(&self, dust_limit: &BigUint) -> usize {
        self.count_fund_items_by_type(FundType::Active, |fund_item| {
            &fund_item.balance < dust_limit
        })
    }

    /// Unstakes all fund buckets in the waiting list that are below a certin threshold.
    /// Will stop if running low on gas.
    /// Does not block the rest of the contract. If any operation interferes with an interrupted
    /// dust cleanup, the operation can be begun again.
    /// It will auto-reset if the list ends or the current item is no longer valid.
    #[only_owner]
    #[endpoint(dustCleanupWaitingList)]
    fn dust_cleanup_waiting_list(&self, dust_limit: &BigUint) {
        require!(
            !self.is_global_op_in_progress(),
            "contract is temporarily paused as checkpoint is reset"
        );

        self.dust_cleanup_checkpoint().update(|checkpoint| {
            self.swap_dust(
                checkpoint,
                dust_limit,
                FundType::Waiting,
                |_| Some(FundDescription::WithdrawOnly),
                || self.blockchain().get_gas_left() < DUST_GASLIMIT,
            );
        });
    }

    /// Unstakes and unbonds all active fund buckets that are below a certin threshold.
    /// Unlike the regular unstake/unbond process, it will send the funds directly in `WithdrawOnly` state.
    /// Will stop if running low on gas.
    /// Does not block the rest of the contract. If any operation interferes with an interrupted
    /// dust cleanup, the operation can be begun again.
    /// It will auto-reset if the list ends or the current item is no longer valid.
    #[only_owner]
    #[endpoint(dustCleanupActive)]
    fn dust_cleanup_active(&self, dust_limit: &BigUint) {
        require!(
            !self.is_global_op_in_progress(),
            "contract is temporarily paused as checkpoint is reset"
        );

        // reserve half of the gas for the subsequent swap Waiting -> Active
        let reserved_gas = self.blockchain().get_gas_left() / 2 + DUST_GASLIMIT;

        // rewards need to be computed for
        self.compute_one_user_reward(OWNER_USER_ID);

        self.dust_cleanup_checkpoint().update(|checkpoint| {
            self.swap_dust(
                checkpoint,
                dust_limit,
                FundType::Active,
                |fund_item| {
                    if let Some(user_id_nz) = NonZeroUsize::new(fund_item.user_id) {
                        self.compute_one_user_reward(user_id_nz);
                        Some(FundDescription::UnStaked { created: 0 })
                    } else {
                        None
                    }
                },
                || self.blockchain().get_gas_left() < reserved_gas,
            );
        });

        // move funds around
        self.use_waiting_to_replace_unstaked();
    }
}
