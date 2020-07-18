imports!();

use crate::fund_module::*;
use super::fund_item::*;
use super::fund_type::*;


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
                fund_type: FundType::Free{ requested_unstake: false },
                user_id,
            },
            balance.clone()
        );
    }

    fn liquidate_free_stake(&self, user_id: usize, amount: &mut BigUint) {
        // first withdraw from withdraw-only inactive stake
        self.fund_module().destroy_max(
            amount,
            DISCR_FREE,
            |fund_info|
                fund_info.user_id == user_id &&
                if let FundType::Free{ requested_unstake: true } = fund_info.fund_type { true } else { false }
        );
        // if that is not enough, retrieve proper inactive stake
        if *amount > 0 {
            self.fund_module().destroy_max(
                amount,
                DISCR_FREE,
                |fund_info|
                    fund_info.user_id == user_id &&
                    if let FundType::Free{ requested_unstake: false } = fund_info.fund_type { true } else { false }
            );
        }
    }

    fn activate_start_transf(&self, amount: &mut BigUint) -> SCResult<()> {
        self.fund_module().split_convert_max(
            amount,
            DISCR_FREE,
            DISCR_PENDING_ACT,
            |fund_info| {
                if let FundType::Free{ requested_unstake: false } = fund_info.fund_type {
                    return Some(FundInfo {
                        fund_type: FundType::PendingActivation,
                        user_id: fund_info.user_id,
                    })
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
                if let FundType::PendingActivation = fund_info.fund_type {
                    return Some(FundInfo {
                        fund_type: FundType::Active,
                        user_id: fund_info.user_id,
                    })
                }
                None
            }
        );
        if *amount > 0 {
            return sc_error!("cannot offer more than the user active stake");
        }

        Ok(())
    }

    fn activate_finish_fail_transf(&self, amount: &mut BigUint) -> SCResult<()> {
        self.fund_module().split_convert_max(
            amount,
            DISCR_PENDING_ACT,
            DISCR_ACTIVE_FAILED,
            |fund_info| {
                if let FundType::PendingActivation = fund_info.fund_type {
                    return Some(FundInfo {
                        fund_type: FundType::ActivationFailed,
                        user_id: fund_info.user_id,
                    })
                }
                None
            }
        );
        if *amount > 0 {
            return sc_error!("cannot offer more than the user active stake");
        }

        Ok(())
    }

    fn announce_unstake_transf(&self, seller_id: usize, amount: &BigUint) -> SCResult<()> {
        let mut amount_to_unstake = amount.clone();
        let current_bl_nonce = self.get_block_nonce();
        self.fund_module().split_convert_max(
            &mut amount_to_unstake,
            DISCR_ACTIVE,
            DISCR_ACTIVE_FOR_SALE,
            |fund_info| {
                if let FundType::Active = fund_info.fund_type {
                    if fund_info.user_id == seller_id {
                        return Some(FundInfo {
                            fund_type: FundType::ActiveForSale{ created: current_bl_nonce },
                            user_id: seller_id,
                        })
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

    fn stake_sale_transf(&self, buyer_id: usize, seller_id: usize, amount: &BigUint) -> SCResult<()> {
        // convert stake
        let mut stake_to_convert = amount.clone();
        self.fund_module().split_convert_max(
            &mut stake_to_convert,
            DISCR_ACTIVE_FOR_SALE,
            DISCR_ACTIVE,
            |fund_info| {
                if let FundType::ActiveForSale{ .. } = fund_info.fund_type {
                    if fund_info.user_id == seller_id {
                        return Some(FundInfo {
                            fund_type: FundType::Active,
                            user_id: buyer_id,
                        })
                    }
                }
                None
            }
        );
        if stake_to_convert > 0 {
            return sc_error!("not enough stake for sale");
        }

        // convert payment
        let mut payment_to_convert = amount.clone();
        let current_bl_nonce = self.get_block_nonce();
        self.fund_module().split_convert_max(
            &mut payment_to_convert,
            DISCR_FREE,
            DISCR_DEF_PAYMENT,
            |fund_info| {
                if let FundType::Free{ .. } = fund_info.fund_type {
                    if fund_info.user_id == buyer_id {
                        return Some(FundInfo {
                            fund_type: FundType::DeferredPayment{ created: current_bl_nonce },
                            user_id: seller_id,
                        })
                    }
                }
                None
            }
        );
        if payment_to_convert > 0 {
            return sc_error!("not enough funds for payment");
        }

        Ok(())
    }

    fn eligible_deferred_payment(&self, 
        user_id: usize, 
        n_blocks_before_claim: u64) -> BigUint {

        let current_bl_nonce = self.get_block_nonce();
        self.fund_module().query_list(
            DISCR_DEF_PAYMENT,
            |fund_info| {
                if let FundType::DeferredPayment{ created } = fund_info.fund_type {
                    if fund_info.user_id == user_id && 
                        current_bl_nonce > created + n_blocks_before_claim {
                        return true;
                    }
                }
                false
            }
        )
    }

    fn claim_all_eligible_deferred_payment(&self,
        user_id: usize,
        n_blocks_before_claim: u64) -> SCResult<BigUint> {
        
        let current_bl_nonce = self.get_block_nonce();
        self.fund_module().split_convert_all(
            DISCR_DEF_PAYMENT,
            DISCR_FREE,
            |fund_info| {
                if let FundType::DeferredPayment{ created } = fund_info.fund_type {
                    if fund_info.user_id == user_id && 
                       current_bl_nonce > created + n_blocks_before_claim {
                        return Some(FundInfo {
                            fund_type: FundType::Free{ requested_unstake: true },
                            user_id: fund_info.user_id,
                        })
                    }
                }
                None
            }
        )
    }

    fn eligible_for_unstake(&self, 
        user_id: usize, 
        n_blocks_before_force_unstake: u64) -> BigUint {

        let current_bl_nonce = self.get_block_nonce();
        self.fund_module().query_list(
            DISCR_ACTIVE_FOR_SALE,
            |fund_info| {
                if let FundType::ActiveForSale{ created } = fund_info.fund_type {
                    if fund_info.user_id == user_id && 
                        current_bl_nonce > created + n_blocks_before_force_unstake {
                        return true;
                    }
                }
                false
            }
        )
    }

    fn unstake_start_transf(&self, 
        opt_requester: Option<usize>, 
        n_blocks_before_force_unstake: u64, 
        amount: &mut BigUint) -> SCResult<()> {

        let mut amount_to_unstake = amount.clone();
        let current_bl_nonce = self.get_block_nonce();

        // 1: unstake requester's stake (if applicable)
        if let Some(requester_id) = opt_requester {
            self.fund_module().split_convert_max(
                &mut amount_to_unstake,
                DISCR_ACTIVE_FOR_SALE,
                DISCR_PENDING_DEACT,
                |fund_info| {
                    if let FundType::ActiveForSale{ created } = fund_info.fund_type {
                        if fund_info.user_id == requester_id && 
                           current_bl_nonce > created + n_blocks_before_force_unstake {
                            return Some(FundInfo {
                                fund_type: FundType::PendingDeactivation{ requested_unstake: true },
                                user_id: fund_info.user_id,
                            })
                        }
                    }
                    None
                }
            );

            if amount_to_unstake == 0 {
                return Ok(());
            }
        }

        // 2: unstake any stake for sale
        // (here we don't take into consideration n_blocks_before_force_unstake,
        // but stake put up for sale longer ago comes first,
        // so naturally the eligible stake for sale will come before the pending one)
        self.fund_module().split_convert_max(
            &mut amount_to_unstake,
            DISCR_ACTIVE_FOR_SALE,
            DISCR_PENDING_DEACT,
            |fund_info| {
                if let FundType::ActiveForSale{ .. } = fund_info.fund_type {
                    return Some(FundInfo {
                        fund_type: FundType::PendingDeactivation{ requested_unstake: true },
                        user_id: fund_info.user_id,
                    })
                }
                None
            }
        );

        if amount_to_unstake == 0 {
            return Ok(());
        }        

        // 3: unstake active stake
        self.fund_module().split_convert_max(
            &mut amount_to_unstake,
            DISCR_ACTIVE,
            DISCR_PENDING_DEACT,
            |fund_info| {
                if let FundType::Active = fund_info.fund_type {
                    return Some(FundInfo {
                        fund_type: FundType::PendingDeactivation{ requested_unstake: false },
                        user_id: fund_info.user_id,
                    })
                }
                None
            }
        );

        if amount_to_unstake == 0 {
            return Ok(());
        }

        sc_error!("insufficient stake to unstake")
    }

    fn unstake_finish_ok_transf(&self, amount: &mut BigUint) -> SCResult<()> {
        let current_bl_nonce = self.get_block_nonce();
        self.fund_module().split_convert_max(
            amount,
            DISCR_PENDING_DEACT,
            DISCR_UNBOND,
            |fund_info| {
                if let FundType::PendingDeactivation { requested_unstake } = fund_info.fund_type {
                    return Some(FundInfo {
                        fund_type: FundType::UnBondPeriod {
                            created: current_bl_nonce,
                            requested_unstake,
                        },
                        user_id: fund_info.user_id,
                    })
                }
                None
            }
        );
        if *amount > 0 {
            return sc_error!("cannot offer more than the user active stake");
        }

        Ok(())
    }

    fn unstake_finish_fail_transf(&self, amount: &mut BigUint) -> SCResult<()> {
        // TODO: also revert to ActiveForSale
        self.fund_module().split_convert_max(
            amount,
            DISCR_PENDING_DEACT,
            DISCR_ACTIVE,
            |fund_info| {
                if let FundType::PendingDeactivation { .. } = fund_info.fund_type {
                    return Some(FundInfo {
                        fund_type: FundType::Active,
                        user_id: fund_info.user_id,
                    })
                }
                None
            }
        );
        if *amount > 0 {
            return sc_error!("cannot offer more than the user active stake");
        }

        Ok(())
    }

    fn unbond_start_transf(&self,
        n_blocks_before_unbond: u64,
        amount: &mut BigUint) -> SCResult<()> {

        let current_bl_nonce = self.get_block_nonce();
        self.fund_module().split_convert_max(
            amount,
            DISCR_UNBOND,
            DISCR_PENDING_UNBOND,
            |fund_info| {
                if let FundType::UnBondPeriod { created, requested_unstake } = fund_info.fund_type {
                    if current_bl_nonce >= created + n_blocks_before_unbond {
                        return Some(FundInfo {
                            fund_type: FundType::PendingUnBond {
                                unbond_created: created,
                                requested_unstake,
                            },
                            user_id: fund_info.user_id,
                        });
                    }
                }
                None
            }
        );
        if *amount > 0 {
            return sc_error!("cannot offer more than the user active stake");
        }

        Ok(())
    }

    fn unbond_finish_ok_transf(&self, amount: &mut BigUint) -> SCResult<()> {
        // let current_bl_nonce = self.get_block_nonce();
        self.fund_module().split_convert_max(
            amount,
            DISCR_PENDING_UNBOND,
            DISCR_FREE,
            |fund_info| {
                if let FundType::PendingUnBond { unbond_created: _, requested_unstake } = fund_info.fund_type {
                    return Some(FundInfo {
                        fund_type: FundType::Free {
                            requested_unstake,
                        },
                        user_id: fund_info.user_id,
                    });
                }
                None
            }
        );
        if *amount > 0 {
            return sc_error!("cannot offer more than the user active stake");
        }

        Ok(())
    }

    fn unbond_finish_fail_transf(&self, amount: &mut BigUint) -> SCResult<()> {
        self.fund_module().split_convert_max(
            amount,
            DISCR_PENDING_UNBOND,
            DISCR_UNBOND,
            |fund_info| {
                if let FundType::PendingUnBond { unbond_created, requested_unstake } = fund_info.fund_type {
                    return Some(FundInfo {
                        fund_type: FundType::UnBondPeriod {
                            created: unbond_created,
                            requested_unstake,
                        },
                        user_id: fund_info.user_id,
                    });
                }
                None
            }
        );
        if *amount > 0 {
            return sc_error!("cannot offer more than the user active stake");
        }

        Ok(())
    }

    fn claim_activation_failed_transf(&self, amount: &mut BigUint) -> SCResult<()> {
        self.fund_module().split_convert_max(
            amount,
            DISCR_ACTIVE_FAILED,
            DISCR_FREE,
            |fund_info| {
                if let FundType::ActivationFailed = fund_info.fund_type {
                    return Some(FundInfo {
                        fund_type: FundType::Free {
                            requested_unstake: false,
                        },
                        user_id: fund_info.user_id,
                    });
                }
                None
            }
        );
        // if *amount > 0 {
        //     return sc_error!("cannot offer more than the user active stake");
        // }

        Ok(())
    }
}
