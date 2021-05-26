#![no_std]
#![allow(clippy::string_lit_as_bytes)]

#[cfg(feature = "delegation_latest_default")]
pub use delegation_latest_default as delegation_latest;
#[cfg(feature = "delegation_latest_wasm")]
pub use delegation_latest_wasm as delegation_latest;

use delegation_latest::node_storage::types::BLSStatusMultiArg;
use delegation_latest::settings::{OWNER_USER_ID, PERCENTAGE_DENOMINATOR};

elrond_wasm::imports!();

#[elrond_wasm_derive::contract]
pub trait DelegationFull:
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
    + delegation_latest::elrond_wasm_module_features::FeaturesModule
    + delegation_latest::elrond_wasm_module_pause::PauseModule
    + delegation_latest::reset_checkpoint_endpoints::ResetCheckpointsModule
    + delegation_latest::rewards_endpoints::RewardEndpointsModule
    + delegation_latest::user_stake_endpoints::UserStakeEndpointsModule
{
    // METADATA

    #[endpoint]
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    // INIT

    /// This is the contract constructor, called only once when the contract is deployed.
    #[init]
    fn init(
        &self,
        auction_contract_addr: &Address,
        service_fee_per_10000: usize,
        owner_min_stake_share_per_10000: usize,
        n_blocks_before_unbond: u64,
        minimum_stake: Self::BigUint,
        total_delegation_cap: Self::BigUint,
    ) -> SCResult<()> {
        let owner = self.blockchain().get_caller();
        self.set_user_id(&owner, OWNER_USER_ID.get()); // node reward destination will be user #1
        self.set_user_address(OWNER_USER_ID.get(), &owner);
        self.set_num_users(1);

        self.set_auction_addr(&auction_contract_addr);

        require!(
            service_fee_per_10000 <= PERCENTAGE_DENOMINATOR,
            "service fee out of range"
        );

        let next_service_fee = Self::BigUint::from(service_fee_per_10000);
        self.set_service_fee(next_service_fee);

        self.set_owner_min_stake_share_validated(owner_min_stake_share_per_10000)?;

        self.set_n_blocks_before_unbond(n_blocks_before_unbond);
        self.set_minimum_stake(&minimum_stake);

        self.set_total_delegation_cap(total_delegation_cap);
        self.set_bootstrap_mode(true);

        Ok(())
    }

    // Callbacks can only be declared here for the moment.

    #[callback(auction_stake_callback)]
    fn auction_stake_callback_root(
        &self,
        node_ids: Vec<usize>,
        #[call_result] call_result: AsyncCallResult<MultiResultVec<BLSStatusMultiArg>>,
    ) -> SCResult<()> {
        self.auction_stake_callback(node_ids, call_result)
    }

    #[callback(auction_unstake_callback)]
    fn auction_unstake_callback_root(
        &self,
        node_ids: Vec<usize>,
        #[call_result] call_result: AsyncCallResult<MultiResultVec<BLSStatusMultiArg>>,
    ) -> SCResult<()> {
        self.auction_unstake_callback(node_ids, call_result)
    }

    #[callback(auction_unbond_callback)]
    fn auction_unbond_callback_root(
        &self,
        node_ids: Vec<usize>,
        #[call_result] call_result: AsyncCallResult<MultiResultVec<BLSStatusMultiArg>>,
    ) -> SCResult<()> {
        self.auction_unbond_callback(node_ids, call_result)
    }
}
