#![no_std]
#![allow(unused_attributes)]
#![allow(clippy::string_lit_as_bytes)]

#[cfg(feature = "delegation_latest_default")]
pub use delegation_latest_default as delegation_latest;
#[cfg(feature = "delegation_latest_wasm")]
pub use delegation_latest_wasm as delegation_latest;

use delegation_latest::*;

elrond_wasm::imports!();

#[elrond_wasm_derive::contract(DelegationImpl)]
pub trait Delegation {
    // METADATA

    #[endpoint]
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    // MODULES

    #[module(EventsModuleImpl)]
    fn events(&self) -> EventsModuleImpl<T, BigInt, BigUint>;

    #[module(NodeConfigModuleImpl)]
    fn node_config(&self) -> NodeConfigModuleImpl<T, BigInt, BigUint>;

    #[module(RewardsModuleImpl)]
    fn rewards(&self) -> RewardsModuleImpl<T, BigInt, BigUint>;

    #[module(ResetCheckpointsModuleImpl)]
    fn reset_checkpoints(&self) -> ResetCheckpointsModuleImpl<T, BigInt, BigUint>;

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    #[module(PauseModuleImpl)]
    fn pause(&self) -> PauseModuleImpl<T, BigInt, BigUint>;

    #[module(UserStakeModuleImpl)]
    fn user_stake(&self) -> UserStakeModuleImpl<T, BigInt, BigUint>;

    #[module(NodeActivationModuleImpl)]
    fn node_activation(&self) -> NodeActivationModuleImpl<T, BigInt, BigUint>;

    #[module(UserUnStakeModuleImpl)]
    fn user_unstake(&self) -> UserUnStakeModuleImpl<T, BigInt, BigUint>;

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(FundTransformationsModuleImpl)]
    fn fund_transf_module(&self) -> FundTransformationsModuleImpl<T, BigInt, BigUint>;

    #[module(FundViewModuleImpl)]
    fn fund_view_module(&self) -> FundViewModuleImpl<T, BigInt, BigUint>;

    // INIT - update from genesis version

    /// the genesis contract didn't have the concept of total delegation cap
    /// so the field needs to be updated here to correspond to how much was staked
    /// the change happened in 0.5.0, but it is still here, so it is not missed in case an upgrade was skipped
    /// checking that the total delegation cap was not already set
    fn update_total_delegation_cap_if_necessary(&self) {
        if self.settings().get_total_delegation_cap() == 0 {
            let total_active = self
                .fund_view_module()
                .get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active);
            // there was no unstaked stake when the 0.5.0 upgrade happened, but the correct formula includes it
            // in some edge cases on the testnets, unstaking and then upgrading can potentially lead to invariant violations
            let total_unstaked = self
                .fund_view_module()
                .get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::UnStaked);
            self.settings()
                .set_total_delegation_cap(total_active + total_unstaked);
        }
    }

    #[init]
    fn init(&self) -> SCResult<()> {
        self.update_total_delegation_cap_if_necessary();
        Ok(())
    }

    // Callbacks can only be declared here for the moment.

    #[callback]
    fn auction_stake_callback(
        &self,
        node_ids: Vec<usize>,
        #[call_result] call_result: AsyncCallResult<MultiResultVec<BLSStatusMultiArg>>,
    ) {
        self.node_activation()
            .auction_stake_callback(node_ids, call_result)
            .unwrap();
        // TODO: replace unwrap with typical Result handling
    }

    #[callback]
    fn auction_unstake_callback(
        &self,
        node_ids: Vec<usize>,
        #[call_result] call_result: AsyncCallResult<MultiResultVec<BLSStatusMultiArg>>,
    ) {
        self.node_activation()
            .auction_unstake_callback(node_ids, call_result)
            .unwrap();
        // TODO: replace unwrap with typical Result handling
    }

    #[callback]
    fn auction_unbond_callback(
        &self,
        node_ids: Vec<usize>,
        #[call_result] call_result: AsyncCallResult<MultiResultVec<BLSStatusMultiArg>>,
    ) {
        self.node_activation()
            .auction_unbond_callback(node_ids, call_result)
            .unwrap();
        // TODO: replace unwrap with typical Result handling
    }
}
