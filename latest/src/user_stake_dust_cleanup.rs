use user_fund_storage::types::FundType;

elrond_wasm::imports!();

pub const DUST_GASLIMIT: u64 = 20_000_000;

/// Functionality for cleaning up very small amounts left in the waiting list.
#[elrond_wasm_derive::module]
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
    fn dust_cleanup_checkpoint(&self) -> SingleValueMapper<Self::Storage, usize>;

    /// Counts fund buckets in the waiting list that are below a certain threshold.
    /// Unlike most views, yields the number of entries, rather than the sum of EGLD.
    #[view(countDustItemsWaitingList)]
    fn count_dust_items_waiting_list(&self, dust_limit: &Self::BigUint) -> usize {
        self.count_fund_items_by_type(FundType::Waiting, |fund_item| {
            &fund_item.balance < dust_limit
        })
    }

    /// Counts fund buckets in the active staking list that are below a certain threshold.
    /// Unlike most views, yields the number of entries, rather than the sum of EGLD.
    #[view(countDustItemsActive)]
    fn count_dust_items_active(&self, dust_limit: &Self::BigUint) -> usize {
        self.count_fund_items_by_type(FundType::Active, |fund_item| {
            &fund_item.balance < dust_limit
        })
    }

    /// Unstakes all fund buckets in the waiting list that are below a certin threshold.
    /// Will stop if running low on gas.
    /// Does not block the rest of the contract. If any operation interferes with an interrupted
    /// dust cleanup, the operation can be begun again.
    /// It will auto-reset if the list ends or the current item is no longer valid.
    #[endpoint(dustCleanupWaitingList)]
    fn dust_cleanup_waiting_list(&self, dust_limit: &Self::BigUint) -> SCResult<()> {
        only_owner!(self, "only owner allowed to clean up dust");

        require!(
            !self.is_global_op_in_progress(),
            "unstaking is temporarily paused as checkpoint is reset"
        );

        self.dust_cleanup_checkpoint().update(|checkpoint| {
            self.swap_dust_to_withdraw_only(checkpoint, FundType::Waiting, dust_limit, || {
                self.blockchain().get_gas_left() < DUST_GASLIMIT
            });
        });

        Ok(())
    }

    /// Unstakes and unbonds all active fund buckets that are below a certin threshold.
    /// Unlike the regular unstake/unbond process, it will send the funds directly in `WithdrawOnly` state.
    /// Will stop if running low on gas.
    /// Does not block the rest of the contract. If any operation interferes with an interrupted
    /// dust cleanup, the operation can be begun again.
    /// It will auto-reset if the list ends or the current item is no longer valid.
    #[endpoint(dustCleanupActive)]
    fn dust_cleanup_active(&self, dust_limit: &Self::BigUint) -> SCResult<()> {
        only_owner!(self, "only owner allowed to clean up dust");

        require!(
            !self.is_global_op_in_progress(),
            "unstaking is temporarily paused as checkpoint is reset"
        );

        // reserve half of the gas for the subsequent swap Waiting -> Active
        let reserved_gas = self.blockchain().get_gas_left() / 2 + DUST_GASLIMIT;

        self.dust_cleanup_checkpoint().update(|checkpoint| {
            self.swap_dust_to_withdraw_only(checkpoint, FundType::Active, dust_limit, || {
                self.blockchain().get_gas_left() < reserved_gas
            });
        });

        // move funds around
        self.use_waiting_to_replace_unstaked()?;

        Ok(())
    }
}
