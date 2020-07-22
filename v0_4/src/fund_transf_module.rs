imports!();

use crate::fund_module::*;
use crate::types::fund_item::*;
use crate::types::fund_type::*;


/// Deals with storage data about delegators.
#[elrond_wasm_derive::module(FundTransformationsModuleImpl)]
pub trait FundTransformationsModule {

    #[module(FundModuleImpl)]
    fn fund_module(&self) -> FundModuleImpl<T, BigInt, BigUint>;

    #[module(FundTransformationsModuleImpl)]
    fn fund_transf_module(&self) -> FundTransformationsModuleImpl<T, BigInt, BigUint>;

    fn create_free_stake(&self, user_id: usize, balance: &BigUint) {
        self.fund_module().create_fund(
            FundInfo{
                fund_desc: FundDescription::Inactive,
                user_id,
            },
            balance.clone()
        );
    }

    fn liquidate_free_stake(&self, user_id: usize, amount: &mut BigUint) {
        // first withdraw from withdraw-only inactive stake
        self.fund_module().destroy_max(
            amount,
            DISCR_WITHDRAW_ONLY,
            |fund_info| fund_info.user_id == user_id 
        );
        // if that is not enough, retrieve proper inactive stake
        if *amount > 0 {
            self.fund_module().destroy_max(
                amount,
                DISCR_INACTIVE,
                |fund_info| fund_info.user_id == user_id
            );
        }
    }

    fn activate_start_transf(&self, amount: &mut BigUint) -> SCResult<()> {
        self.fund_module().split_convert_max(
            amount,
            DISCR_INACTIVE,
            DISCR_PENDING_ACT,
            |fund_info| {
                if let FundDescription::Inactive = fund_info.fund_desc {
                    return Some(FundDescription::PendingActivation)
                }
                None
            }
        );
        if *amount > 0 {
            return sc_error!("not enough inactive stake");
        }

        Ok(())
    }

    fn activate_finish_ok_transf(&self, amount: &mut BigUint) -> SCResult<()> {
        self.fund_module().split_convert_max(
            amount,
            DISCR_PENDING_ACT,
            DISCR_ACTIVE,
            |fund_info| {
                if let FundDescription::PendingActivation = fund_info.fund_desc {
                    return Some(FundDescription::Active)
                }
                None
            }
        );
        if *amount > 0 {
            return sc_error!("not enough stake pending activation");
        }

        Ok(())
    }

    fn activate_finish_fail_transf(&self, amount: &mut BigUint) -> SCResult<()> {
        self.fund_module().split_convert_max(
            amount,
            DISCR_PENDING_ACT,
            DISCR_ACTIVE_FAILED,
            |fund_info| {
                if let FundDescription::PendingActivation = fund_info.fund_desc {
                    return Some(FundDescription::ActivationFailed)
                }
                None
            }
        );
        if *amount > 0 {
            return sc_error!("not enough stake pending activation");
        }

        Ok(())
    }

    fn unstake_transf(&self, seller_id: usize, amount: &BigUint) -> SCResult<()> {
        let mut amount_to_unstake = amount.clone();
        let current_bl_nonce = self.get_block_nonce();
        self.fund_module().split_convert_max(
            &mut amount_to_unstake,
            DISCR_ACTIVE,
            DISCR_UNSTAKED,
            |fund_info| {
                if let FundDescription::Active = fund_info.fund_desc {
                    if fund_info.user_id == seller_id {
                        return Some(FundDescription::UnStaked{ created: current_bl_nonce })
                    }
                }
                None
            }
        );
        if amount_to_unstake > 0 {
            return sc_error!("cannot offer more than the user active stake");
        }

        Ok(())
    }

    fn unstake_swap_transf(&self, unstake_user_id: usize, amount: &BigUint) -> SCResult<()> {
        // convert active stake -> deferred payment (seller)
        let mut unstaked_to_convert = amount.clone();
        let current_bl_nonce = self.get_block_nonce();
        self.fund_module().split_convert_max(
            &mut unstaked_to_convert,
            DISCR_UNSTAKED,
            DISCR_DEF_PAYMENT,
            |fund_info| {
                if let FundDescription::UnStaked{ .. } = fund_info.fund_desc {
                    if fund_info.user_id == unstake_user_id {
                        return Some(FundDescription::DeferredPayment{ created: current_bl_nonce })
                    }
                }
                None
            }
        );

        // convert inactive -> active (buyer)
        let mut stake_to_activate = amount.clone();
        self.fund_module().split_convert_max(
            &mut stake_to_activate,
            DISCR_INACTIVE,
            DISCR_ACTIVE,
            |fund_info| {
                if let FundDescription::Inactive = fund_info.fund_desc {
                    return Some(FundDescription::Active)
                }
                None
            }
        );

        Ok(())
    }

    fn eligible_deferred_payment(&self, 
        user_id: usize, 
        n_blocks_before_claim: u64) -> BigUint {

        let current_bl_nonce = self.get_block_nonce();
        self.fund_module().query_list(
            DISCR_DEF_PAYMENT,
            |fund_info| {
                if let FundDescription::DeferredPayment{ created } = fund_info.fund_desc {
                    if fund_info.user_id == user_id && 
                        current_bl_nonce > created + n_blocks_before_claim {
                        return true;
                    }
                }
                false
            }
        )
    }

    fn claim_all_eligible_deferred_payments(&self,
        user_id: usize,
        n_blocks_before_claim: u64) -> SCResult<BigUint> {
        
        let current_bl_nonce = self.get_block_nonce();
        self.fund_module().split_convert_all(
            DISCR_DEF_PAYMENT,
            DISCR_INACTIVE,
            |fund_info| {
                if let FundDescription::DeferredPayment{ created } = fund_info.fund_desc {
                    if fund_info.user_id == user_id && 
                       current_bl_nonce > created + n_blocks_before_claim {
                        return Some(FundInfo {
                            fund_desc: FundDescription::WithdrawOnly,
                            user_id: fund_info.user_id,
                        })
                    }
                }
                None
            }
        )
    }

    fn node_unbond_transf(&self, amount: &mut BigUint) -> SCResult<()> {
        self.fund_module().split_convert_max(
            amount,
            DISCR_UNSTAKED,
            DISCR_WITHDRAW_ONLY,
            |fund_info| {
                if let FundDescription::UnStaked { created: _ } = fund_info.fund_desc {
                    return Some(FundDescription::WithdrawOnly);
                }
                None
            }
        );
        if *amount > 0 {
            self.fund_module().split_convert_max(
                amount,
                DISCR_ACTIVE,
                DISCR_INACTIVE,
                |fund_info| {
                    if let FundDescription::Active = fund_info.fund_desc {
                        return Some(FundDescription::Inactive);
                    }
                    None
                }
            );
        }

        Ok(())
    }

    fn claim_activation_failed_transf(&self, amount: &mut BigUint) -> SCResult<()> {
        self.fund_module().split_convert_max(
            amount,
            DISCR_ACTIVE_FAILED,
            DISCR_INACTIVE,
            |fund_info| {
                if let FundDescription::ActivationFailed = fund_info.fund_desc {
                    return Some(FundDescription::Inactive);
                }
                None
            }
        );
        if *amount > 0 {
            return sc_error!("not enough stake activation failed");
        }

        Ok(())
    }
}
