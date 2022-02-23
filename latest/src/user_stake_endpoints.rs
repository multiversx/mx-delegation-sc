use crate::settings::OWNER_USER_ID;
use core::num::NonZeroUsize;
use user_fund_storage::types::FundType;

elrond_wasm::imports!();

pub const UNBOND_GASLIMIT: u64 = 50_000_000;

#[elrond_wasm::derive::module]
pub trait UserStakeEndpointsModule:
    crate::user_stake_state::UserStakeStateModule
    + crate::reset_checkpoint_state::ResetCheckpointStateModule
    + crate::rewards_state::RewardStateModule
    + crate::settings::SettingsModule
    + crate::events::EventsModule
    + user_fund_storage::user_data::UserDataModule
    + user_fund_storage::fund_module::FundModule
    + user_fund_storage::fund_view_module::FundViewModule
    + user_fund_storage::fund_transf_module::FundTransformationsModule
    + elrond_wasm_modules::features::FeaturesModule
    + elrond_wasm_modules::pause::PauseModule
{
    /// Delegate stake to the smart contract.
    /// Stake is initially inactive, so does it not produce rewards.
    #[payable("EGLD")]
    #[endpoint(stake)]
    fn stake_endpoint(&self, #[payment] payment: BigUint) {
        require!(self.not_paused(), "contract paused");

        require!(
            payment >= self.get_minimum_stake(),
            "cannot stake less than minimum stake"
        );

        require!(
            !self.is_global_op_in_progress(),
            "staking is temporarily paused as checkpoint is reset"
        );

        self.process_stake(payment)
    }

    /// unStake - the user will announce that he wants to get out of the contract
    /// selected funds will change from active to inactive, but claimable only after unBond period ends
    #[endpoint(unStake)]
    fn unstake_endpoint(&self, amount: BigUint) {
        require!(self.not_paused(), "contract paused");

        require!(
            !self.is_global_op_in_progress(),
            "unstaking is temporarily paused as checkpoint is reset"
        );

        let caller = self.blockchain().get_caller();
        let unstake_user_id = NonZeroUsize::new(self.get_user_id(&caller))
            .unwrap_or_else(|| sc_panic!("only delegators can unstake"));

        // validate that amount does not exceed existing waiting + active stake
        self.validate_unstake_amount(unstake_user_id.get(), &amount);

        // first try to remove funds from waiting list
        let mut remaining = amount;
        self.swap_user_waiting_to_withdraw_only(unstake_user_id.get(), &mut remaining);
        if remaining == 0 {
            // waiting list entries covered the whole sum
            return;
        }

        // compute rewards before converting Active -> UnStaked
        self.compute_one_user_reward(OWNER_USER_ID);
        self.compute_one_user_reward(unstake_user_id);

        // convert Active -> UnStaked
        self.swap_user_active_to_unstaked(unstake_user_id.get(), &mut remaining);
        require!(remaining == 0, "error converting Active to UnStaked");

        // move funds around
        self.use_waiting_to_replace_unstaked();

        // check that minimum stake was not violated
        self.validate_user_minimum_stake(unstake_user_id.get());
    }

    #[view(getUnStakeable)]
    fn get_unstakeable(&self, user_address: ManagedAddress) -> BigUint {
        let user_id = self.get_user_id(&user_address);
        if user_id == 0 {
            BigUint::zero()
        } else {
            self.get_user_stake_of_type(user_id, FundType::Waiting)
                + self.get_user_stake_of_type(user_id, FundType::Active)
        }
    }

    #[endpoint(unBond)]
    fn unbond_user(&self) -> BigUint {
        require!(self.not_paused(), "contract paused");

        let caller = self.blockchain().get_caller();
        let caller_id = self.get_user_id(&caller);
        require!(caller_id > 0, "unknown caller");

        let n_blocks_before_unbond = self.get_n_blocks_before_unbond();
        let _ = self.swap_eligible_deferred_to_withdraw(caller_id, n_blocks_before_unbond, || {
            self.blockchain().get_gas_left() < UNBOND_GASLIMIT
        });

        let amount_liquidated = self.liquidate_all_withdraw_only(caller_id, || {
            self.blockchain().get_gas_left() < UNBOND_GASLIMIT
        });

        if amount_liquidated > 0 {
            // forward payment to seller
            self.send()
                .direct_egld(&caller, &amount_liquidated, b"delegation stake unbond");
        }

        amount_liquidated
    }

    #[view(getUnBondable)]
    fn get_unbondable(&self, user_address: ManagedAddress) -> BigUint {
        let user_id = self.get_user_id(&user_address);
        if user_id == 0 {
            BigUint::zero()
        } else {
            let n_blocks_before_unbond = self.get_n_blocks_before_unbond();
            self.eligible_deferred_payment(user_id, n_blocks_before_unbond)
                + self.get_user_stake_of_type(user_id, FundType::WithdrawOnly)
        }
    }
}
