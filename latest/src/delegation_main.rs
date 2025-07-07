#![no_std]
#![allow(clippy::string_lit_as_bytes)]
#![allow(clippy::type_complexity)]

// auxiliaries
pub mod auction_proxy;
pub mod governance_sc_proxy;

// modules
pub mod events;
pub mod node_activation;
pub mod reset_checkpoint_endpoints;
pub mod reset_checkpoint_state;
pub mod reset_checkpoint_types;
pub mod rewards_endpoints;
pub mod rewards_state;
pub mod settings;
pub mod user_stake_dust_cleanup;
pub mod user_stake_endpoints;
pub mod user_stake_state;
pub mod governance;

pub use multiversx_sc_modules;
pub use node_storage;
pub use user_fund_storage;

use settings::{OWNER_USER_ID, PERCENTAGE_DENOMINATOR};
use user_fund_storage::fund_view_module::USER_STAKE_TOTALS_ID;
use user_fund_storage::types::FundType;

multiversx_sc::imports!();

#[multiversx_sc::derive::contract]
pub trait DelegationFull:
    node_storage::node_config::NodeConfigModule
    + user_fund_storage::user_data::UserDataModule
    + user_fund_storage::fund_module::FundModule
    + user_fund_storage::fund_view_module::FundViewModule
    + user_fund_storage::fund_transf_module::FundTransformationsModule
    + node_activation::NodeActivationModule
    + settings::SettingsModule
    + reset_checkpoint_state::ResetCheckpointStateModule
    + rewards_state::RewardStateModule
    + user_stake_state::UserStakeStateModule
    + events::EventsModule
    + reset_checkpoint_endpoints::ResetCheckpointsModule
    + rewards_endpoints::RewardEndpointsModule
    + user_stake_endpoints::UserStakeEndpointsModule
    + user_stake_dust_cleanup::UserStakeDustCleanupModule
    + multiversx_sc_modules::dns::DnsModule
    + multiversx_sc_modules::features::FeaturesModule
    + multiversx_sc_modules::pause::PauseModule
    + governance::GovernanceModule
{
    // METADATA

    #[endpoint]
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    // INIT

    /// This is the contract constructor, called only once when the contract is deployed.
    #[init]
    #[label("init")]
    fn init(
        &self,
        auction_contract_addr: &ManagedAddress,
        service_fee_per_10000: usize,
        owner_min_stake_share_per_10000: usize,
        n_blocks_before_unbond: u64,
        minimum_stake: BigUint,
        total_delegation_cap: BigUint,
    ) {
        let owner = self.blockchain().get_caller();
        self.set_user_id(&owner, OWNER_USER_ID.get()); // node reward destination will be user #1
        self.set_user_address(OWNER_USER_ID.get(), &owner);
        self.set_num_users(1);

        self.set_auction_addr(auction_contract_addr);

        require!(
            service_fee_per_10000 <= PERCENTAGE_DENOMINATOR,
            "service fee out of range"
        );

        let next_service_fee = BigUint::from(service_fee_per_10000);
        self.set_service_fee(next_service_fee);

        self.set_owner_min_stake_share_validated(owner_min_stake_share_per_10000);

        self.set_n_blocks_before_unbond(n_blocks_before_unbond);
        self.set_minimum_stake(&minimum_stake);

        self.set_total_delegation_cap(total_delegation_cap);
        self.set_bootstrap_mode(true);
    }

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

    #[upgrade]
    #[label("upgrade")]
    fn upgrade(&self) {
        self.update_total_delegation_cap_if_necessary();
    }
}
