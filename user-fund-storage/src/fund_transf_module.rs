imports!();

use crate::fund_module::*;
use crate::types::fund_type::*;

/// Deals with storage data about delegators.
#[elrond_wasm_derive::module(FundTransformationsModuleImpl)]
pub trait FundTransformationsModule {
    #[module(FundModuleImpl)]
    fn fund_module(&self) -> FundModuleImpl<T, BigInt, BigUint>;

    fn create_waiting(&self, user_id: usize, balance: BigUint) {
        let current_bl_nonce = self.get_block_nonce();
        self.fund_module().increase_fund_balance(
            user_id,
            FundDescription::Waiting {
                created: current_bl_nonce,
            },
            balance,
        );
    }

    fn liquidate_all_withdraw_only(&self, user_id: usize) -> BigUint {
        self.fund_module()
            .destroy_all_for_user(user_id, FundType::WithdrawOnly)
    }

    fn swap_user_active_to_unstaked(&self, unstake_user_id: usize, amount: &mut BigUint) {
        let current_bl_nonce = self.get_block_nonce();
        let _ = self.fund_module().split_convert_max_by_user(
            Some(amount),
            unstake_user_id,
            FundType::Active,
            SwapDirection::Forwards,
            |_| {
                Some(FundDescription::UnStaked {
                    created: current_bl_nonce,
                })
            },
        );
    }

    fn swap_waiting_to_active<I: Fn() -> bool>(
        &self,
        remaining: &mut BigUint,
        interrupt: I,
    ) -> Vec<usize> {
        self.fund_module().split_convert_max_by_type(
            Some(remaining),
            FundType::Waiting,
            SwapDirection::Forwards,
            |_, _| Some(FundDescription::Active),
            interrupt,
            false,
        )
    }

    fn swap_user_waiting_to_withdraw_only(&self, user_id: usize, remaining: &mut BigUint) {
        let _ = self.fund_module().split_convert_max_by_user(
            Some(remaining),
            user_id,
            FundType::Waiting,
            SwapDirection::Backwards,
            |_| Some(FundDescription::WithdrawOnly),
        );
    }

    fn get_affected_users_of_swap_waiting_to_active<I: Fn() -> bool>(
        &self,
        amount: &BigUint,
        interrupt: I,
    ) -> (Vec<usize>, BigUint) {
        let mut stake_to_activate = amount.clone();
        let affected_users = self.fund_module().split_convert_max_by_type(
            Some(&mut stake_to_activate),
            FundType::Waiting,
            SwapDirection::Forwards,
            |_, _| Some(FundDescription::Active),
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
        let _ = self.fund_module().split_convert_max_by_type(
            Some(remaining),
            FundType::UnStaked,
            SwapDirection::Forwards,
            |_, fund_info| match fund_info {
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
        let current_bl_nonce = self.get_block_nonce();
        let _ = self.fund_module().split_convert_max_by_type(
            Some(remaining),
            FundType::Active,
            SwapDirection::Backwards,
            |_, _| {
                Some(FundDescription::DeferredPayment {
                    created: current_bl_nonce,
                })
            },
            interrupt,
            false,
        );
    }

    fn swap_eligible_deferred_to_withdraw(
        &self,
        user_id: usize,
        n_blocks_before_claim: u64,
    ) -> BigUint {
        let current_bl_nonce = self.get_block_nonce();
        self.fund_module().split_convert_max_by_user(
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
        )
    }
}
