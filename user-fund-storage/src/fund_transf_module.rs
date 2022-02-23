elrond_wasm::imports!();

use crate::fund_module;
use crate::fund_module::SwapDirection;
use crate::types::{FundDescription, FundItem, FundType};

/// Deals with storage data about delegators.
#[elrond_wasm::derive::module]
pub trait FundTransformationsModule: fund_module::FundModule {
    fn create_waiting(&self, user_id: usize, balance: BigUint) {
        let current_bl_nonce = self.blockchain().get_block_nonce();
        self.increase_fund_balance(
            user_id,
            FundDescription::Waiting {
                created: current_bl_nonce,
            },
            balance,
        );
    }

    fn liquidate_all_withdraw_only<I: Fn() -> bool>(
        &self,
        user_id: usize,
        interrupt: I,
    ) -> BigUint {
        self.destroy_all_for_user(user_id, FundType::WithdrawOnly, interrupt)
    }

    fn swap_user_active_to_unstaked(&self, unstake_user_id: usize, amount: &mut BigUint) {
        let current_bl_nonce = self.blockchain().get_block_nonce();
        let _ = self.split_convert_max_by_user(
            Some(amount),
            unstake_user_id,
            FundType::Active,
            SwapDirection::Forwards,
            |_| {
                Some(FundDescription::UnStaked {
                    created: current_bl_nonce,
                })
            },
            || false,
        );
    }

    fn swap_waiting_to_active<I: Fn() -> bool>(
        &self,
        remaining: &mut BigUint,
        interrupt: I,
    ) -> ManagedVec<usize> {
        self.split_convert_max_by_type(
            Some(remaining),
            FundType::Waiting,
            SwapDirection::Forwards,
            |_| Some(FundDescription::Active),
            interrupt,
            false,
        )
    }

    fn swap_user_waiting_to_withdraw_only(&self, user_id: usize, remaining: &mut BigUint) {
        let _ = self.split_convert_max_by_user(
            Some(remaining),
            user_id,
            FundType::Waiting,
            SwapDirection::Backwards,
            |_| Some(FundDescription::WithdrawOnly),
            || false,
        );
    }

    /// Applies transformation to all funds below given threshold.
    fn swap_dust<F, I>(
        &self,
        current_id: &mut usize,
        dust_limit: &BigUint,
        source_type: FundType,
        mut filter_transform: F,
        interrupt: I,
    ) where
        F: FnMut(&FundItem<Self::Api>) -> Option<FundDescription>,
        I: Fn() -> bool,
    {
        self.split_convert_max_by_type_with_checkpoint(
            current_id,
            source_type,
            SwapDirection::Backwards,
            |fund_item| {
                if &fund_item.balance < dust_limit {
                    filter_transform(fund_item)
                } else {
                    None
                }
            },
            interrupt,
        );
    }

    fn get_affected_users_of_swap_waiting_to_active<I: Fn() -> bool>(
        &self,
        amount: &BigUint,
        interrupt: I,
    ) -> (ManagedVec<usize>, BigUint) {
        let mut stake_to_activate = amount.clone();
        let affected_users = self.split_convert_max_by_type(
            Some(&mut stake_to_activate),
            FundType::Waiting,
            SwapDirection::Forwards,
            |_| Some(FundDescription::Active),
            interrupt,
            true,
        );

        (affected_users, stake_to_activate)
    }

    fn swap_unstaked_to_deferred_payment<I: Fn() -> bool>(
        &self,
        remaining: &mut BigUint,
        interrupt: I,
    ) {
        let _ = self.split_convert_max_by_type(
            Some(remaining),
            FundType::UnStaked,
            SwapDirection::Forwards,
            |fund_info| match fund_info.fund_desc {
                FundDescription::UnStaked { created } => {
                    Some(FundDescription::DeferredPayment { created })
                }
                _ => None,
            },
            interrupt,
            false,
        );
    }

    fn swap_active_to_deferred_payment<I: Fn() -> bool>(
        &self,
        remaining: &mut BigUint,
        interrupt: I,
    ) {
        let current_bl_nonce = self.blockchain().get_block_nonce();
        let _ = self.split_convert_max_by_type(
            Some(remaining),
            FundType::Active,
            SwapDirection::Backwards,
            |_| {
                Some(FundDescription::DeferredPayment {
                    created: current_bl_nonce,
                })
            },
            interrupt,
            false,
        );
    }

    fn swap_eligible_deferred_to_withdraw<I: Fn() -> bool>(
        &self,
        user_id: usize,
        n_blocks_before_claim: u64,
        interrupt: I,
    ) -> BigUint {
        let current_bl_nonce = self.blockchain().get_block_nonce();
        self.split_convert_max_by_user(
            None,
            user_id,
            FundType::DeferredPayment,
            SwapDirection::Forwards,
            |fund_desc| {
                if let FundDescription::DeferredPayment { created } = fund_desc {
                    if current_bl_nonce >= created + n_blocks_before_claim {
                        return Some(FundDescription::WithdrawOnly);
                    }
                }
                None
            },
            interrupt,
        )
    }
}
