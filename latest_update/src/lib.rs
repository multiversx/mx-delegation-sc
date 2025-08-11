#![no_std]
#![allow(clippy::string_lit_as_bytes)]

use delegation_latest::user_fund_storage::fund_view_module::USER_STAKE_TOTALS_ID;
use delegation_latest::user_fund_storage::types::FundType;

elrond_wasm::imports!();

#[elrond_wasm::derive::contract]
pub trait DelegationUpdate:
    delegation_latest::node_storage::node_config::NodeConfigModule
    + delegation_latest::user_fund_storage::user_data::UserDataModule
    + delegation_latest::user_fund_storage::fund_module::FundModule
    + delegation_latest::user_fund_storage::fund_view_module::FundViewModule
    + delegation_latest::user_fund_storage::fund_transf_module::FundTransformationsModule
    + delegation_latest::node_activation::NodeActivationModule
    + delegation_latest::settings::SettingsModule
    + delegation_latest::reset_checkpoint_state::ResetCheckpointStateModule
    + delegation_latest::rewards_state::RewardStateModule
    + delegation_latest::user_stake_state::UserStakeStateModule
    + delegation_latest::events::EventsModule
    + delegation_latest::reset_checkpoint_endpoints::ResetCheckpointsModule
    + delegation_latest::rewards_endpoints::RewardEndpointsModule
    + delegation_latest::user_stake_endpoints::UserStakeEndpointsModule
    + delegation_latest::user_stake_dust_cleanup::UserStakeDustCleanupModule
    + delegation_latest::elrond_wasm_modules::dns::DnsModule
    + delegation_latest::elrond_wasm_modules::features::FeaturesModule
    + delegation_latest::elrond_wasm_modules::pause::PauseModule
    + delegation_latest::governance::GovernanceModule
{
    // METADATA

    #[endpoint]
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    // INIT - update from genesis version

    /// the genesis contract didn't have the concept of total delegation cap
    /// so the field needs to be updated here to correspond to how much was staked
    /// the change happened in 0.5.0, but it is still here, so it is not missed in case an upgrade was skipped
    /// checking that the total delegation cap was not already set
    fn update_total_delegation_cap_if_necessary(&self) {
        if self.get_total_delegation_cap() == 0 {
            let total_active = self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active);
            // there was no unstaked stake when the 0.5.0 upgrade happened, but the correct formula includes it
            // in some edge cases on the testnets, unstaking and then upgrading can potentially lead to invariant violations
            let total_unstaked =
                self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::UnStaked);
            self.set_total_delegation_cap(total_active + total_unstaked);
        }
    }

    #[init]
    fn init(&self) {}

    #[endpoint]
    fn upgrade(&self) {
        self.update_total_delegation_cap_if_necessary();
    }
}
