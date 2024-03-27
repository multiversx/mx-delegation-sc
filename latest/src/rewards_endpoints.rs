use core::num::NonZeroUsize;

multiversx_sc::imports!();

#[multiversx_sc::derive::module]
pub trait RewardEndpointsModule:
    crate::settings::SettingsModule
    + crate::rewards_state::RewardStateModule
    + crate::reset_checkpoint_state::ResetCheckpointStateModule
    + crate::events::EventsModule
    + user_fund_storage::user_data::UserDataModule
    + user_fund_storage::fund_module::FundModule
    + user_fund_storage::fund_view_module::FundViewModule
    + multiversx_sc_modules::features::FeaturesModule
    + multiversx_sc_modules::pause::PauseModule
{
    /// Retrieve those rewards to which the caller is entitled.
    /// Will send:
    /// - new rewards
    /// - rewards that were previously computed but not sent
    #[endpoint(claimRewards)]
    fn claim_rewards(&self) {
        require!(self.not_paused(), "contract paused");
        self.check_feature_on(b"claimRewards", true);

        let caller = self.blockchain().get_caller();
        let user_id = NonZeroUsize::new(self.get_user_id(&caller))
            .unwrap_or_else(|| sc_panic!("unknown caller"));

        require!(
            !self.is_global_op_in_progress(),
            "claim rewards is temporarily paused as checkpoint is reset"
        );

        let mut user_data = self.load_updated_user_rewards(user_id);

        if user_data.unclaimed_rewards > 0 {
            self.claim_rewards_event(&caller, &user_data.unclaimed_rewards);

            self.send_rewards(&caller, &user_data.unclaimed_rewards);

            user_data.unclaimed_rewards = BigUint::zero();
        }

        self.store_user_reward_data(user_id, &user_data);
    }

    fn send_rewards(&self, to: &ManagedAddress, amount: &BigUint) {
        // send funds
        self.tx().to(to).egld(amount).transfer();

        // increment globally sent funds
        let mut sent_rewards = self.get_sent_rewards();
        sent_rewards += amount;
        self.set_sent_rewards(&sent_rewards);
    }
}
