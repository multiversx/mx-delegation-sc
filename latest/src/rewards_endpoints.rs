use crate::elrond_wasm_module_features::feature_guard;

use core::num::NonZeroUsize;

elrond_wasm::imports!();

#[elrond_wasm_derive::module]
pub trait RewardEndpointsModule:
    crate::settings::SettingsModule
    + crate::rewards_state::RewardStateModule
    + crate::reset_checkpoint_state::ResetCheckpointStateModule
    + crate::events::EventsModule
    + crate::user_fund_storage::user_data::UserDataModule
    + crate::user_fund_storage::fund_module::FundModule
    + crate::user_fund_storage::fund_view_module::FundViewModule
    + crate::elrond_wasm_module_features::FeaturesModule
    + crate::elrond_wasm_module_pause::PauseModule
{
    /// Retrieve those rewards to which the caller is entitled.
    /// Will send:
    /// - new rewards
    /// - rewards that were previously computed but not sent
    #[endpoint(claimRewards)]
    fn claim_rewards(&self) -> SCResult<()> {
        require!(self.not_paused(), "contract paused");
        feature_guard!(self, b"claimRewards", true);

        let caller = self.blockchain().get_caller();
        let user_id = non_zero_usize!(self.get_user_id(&caller), "unknown caller");

        require!(
            !self.is_global_op_in_progress(),
            "claim rewards is temporarily paused as checkpoint is reset"
        );

        let mut user_data = self.load_updated_user_rewards(user_id);

        if user_data.unclaimed_rewards > 0 {
            self.claim_rewards_event(&caller, &user_data.unclaimed_rewards);

            self.send_rewards(&caller, &user_data.unclaimed_rewards);

            user_data.unclaimed_rewards = Self::BigUint::zero();
        }

        self.store_user_reward_data(user_id, &user_data);

        Ok(())
    }

    fn send_rewards(&self, to: &Address, amount: &Self::BigUint) {
        // send funds
        self.send()
            .direct_egld(to, amount, b"delegation rewards claim");

        // increment globally sent funds
        let mut sent_rewards = self.get_sent_rewards();
        sent_rewards += amount;
        self.set_sent_rewards(&sent_rewards);
    }
}
