use user_fund_storage::fund_module::*;
use user_fund_storage::fund_transf_module::*;
use user_fund_storage::types::*;

use elrond_wasm::api::BigUintApi;
use elrond_wasm_debug::api::RustBigUint;
use elrond_wasm_debug::TxContext;

mod fund_module_check;

#[test]
fn test_create_destroy() {
    let transf_module = FundTransformationsModuleImpl::new(TxContext::dummy());
    ();
    let fund_module = transf_module.fund_module();

    let user_id = 2;

    transf_module.create_waiting(user_id, 5000u32.into());

    fund_module_check::check_consistency(&fund_module, 3);
    assert_eq!(
        RustBigUint::from(5000u32),
        fund_module.query_sum_funds_by_type(FundType::Waiting, |_, _| true)
    );
    assert_eq!(
        1,
        fund_module.count_fund_items_by_type(FundType::Waiting, |_, _| true)
    );
    assert_eq!(
        RustBigUint::from(5000u32),
        fund_module.query_sum_funds_by_user_type(user_id, FundType::Waiting, |_| true)
    );
    assert_eq!(
        1,
        fund_module.count_fund_items_by_user_type(user_id, FundType::Waiting, |_| true)
    );

    let mut amount = RustBigUint::from(5000u32);
    transf_module.swap_user_waiting_to_withdraw_only(user_id, &mut amount);
    assert_eq!(amount, RustBigUint::zero());

    let liquidated = transf_module.liquidate_all_withdraw_only(user_id, || false);
    assert_eq!(liquidated, RustBigUint::from(5000u32));

    fund_module_check::check_consistency(&fund_module, 3);

    assert_eq!(
        RustBigUint::from(0u32),
        fund_module.query_sum_funds_by_type(FundType::Waiting, |_, _| true)
    );
    assert_eq!(
        0,
        fund_module.count_fund_items_by_type(FundType::Waiting, |_, _| true)
    );
    assert_eq!(
        RustBigUint::from(0u32),
        fund_module.query_sum_funds_by_user_type(user_id, FundType::Waiting, |_| true)
    );
    assert_eq!(
        0,
        fund_module.count_fund_items_by_user_type(user_id, FundType::Waiting, |_| true)
    );
}

#[test]
fn test_full_cycle_1() {
    let transf_module = FundTransformationsModuleImpl::new(TxContext::dummy());
    ();
    let fund_module = transf_module.fund_module();

    let user_id = 2;

    // create -> Waiting
    transf_module.create_waiting(user_id, 5000u32.into());

    fund_module_check::check_consistency(&fund_module, 3);
    assert_eq!(
        RustBigUint::from(5000u32),
        fund_module.query_sum_funds_by_type(FundType::Waiting, |_, _| true)
    );
    assert_eq!(
        1,
        fund_module.count_fund_items_by_type(FundType::Waiting, |_, _| true)
    );
    assert_eq!(
        RustBigUint::from(5000u32),
        fund_module.query_sum_funds_by_user_type(user_id, FundType::Waiting, |_| true)
    );
    assert_eq!(
        1,
        fund_module.count_fund_items_by_user_type(user_id, FundType::Waiting, |_| true)
    );

    // Waiting -> Active
    let mut amount = RustBigUint::from(5000u32);
    let affected_users = transf_module.swap_waiting_to_active(&mut amount, || false);
    assert_eq!(affected_users, vec![user_id]);
    assert_eq!(amount, RustBigUint::zero());

    fund_module_check::check_consistency(&fund_module, 3);
    assert_eq!(
        RustBigUint::from(0u32),
        fund_module.query_sum_funds_by_type(FundType::Waiting, |_, _| true)
    );
    assert_eq!(
        0,
        fund_module.count_fund_items_by_type(FundType::Waiting, |_, _| true)
    );
    assert_eq!(
        RustBigUint::from(0u32),
        fund_module.query_sum_funds_by_user_type(user_id, FundType::Waiting, |_| true)
    );
    assert_eq!(
        0,
        fund_module.count_fund_items_by_user_type(user_id, FundType::Waiting, |_| true)
    );

    assert_eq!(
        RustBigUint::from(5000u32),
        fund_module.query_sum_funds_by_type(FundType::Active, |_, _| true)
    );
    assert_eq!(
        1,
        fund_module.count_fund_items_by_type(FundType::Active, |_, _| true)
    );
    assert_eq!(
        RustBigUint::from(5000u32),
        fund_module.query_sum_funds_by_user_type(user_id, FundType::Active, |_| true)
    );
    assert_eq!(
        1,
        fund_module.count_fund_items_by_user_type(user_id, FundType::Active, |_| true)
    );

    // Active -> Unstaked
    let mut amount = RustBigUint::from(5000u32);
    transf_module.swap_user_active_to_unstaked(user_id, &mut amount);
    assert_eq!(amount, RustBigUint::zero());

    fund_module_check::check_consistency(&fund_module, 3);
    assert_eq!(
        RustBigUint::from(5000u32),
        fund_module.query_sum_funds_by_type(FundType::UnStaked, |_, _| true)
    );
    assert_eq!(
        1,
        fund_module.count_fund_items_by_type(FundType::UnStaked, |_, _| true)
    );
    assert_eq!(
        RustBigUint::from(5000u32),
        fund_module.query_sum_funds_by_user_type(user_id, FundType::UnStaked, |_| true)
    );
    assert_eq!(
        1,
        fund_module.count_fund_items_by_user_type(user_id, FundType::UnStaked, |_| true)
    );

    // Unstaked -> DeferredPayment
    let mut amount = RustBigUint::from(5000u32);
    transf_module.swap_unstaked_to_deferred_payment(&mut amount, || false);
    assert_eq!(amount, RustBigUint::zero());

    fund_module_check::check_consistency(&fund_module, 3);
    assert_eq!(
        RustBigUint::from(5000u32),
        fund_module.query_sum_funds_by_type(FundType::DeferredPayment, |_, _| true)
    );
    assert_eq!(
        1,
        fund_module.count_fund_items_by_type(FundType::DeferredPayment, |_, _| true)
    );
    assert_eq!(
        RustBigUint::from(5000u32),
        fund_module.query_sum_funds_by_user_type(user_id, FundType::DeferredPayment, |_| true)
    );
    assert_eq!(
        1,
        fund_module.count_fund_items_by_user_type(user_id, FundType::DeferredPayment, |_| true)
    );

    // DeferredPayment -> WithdrawOnly
    let claimed_amount = transf_module.swap_eligible_deferred_to_withdraw(user_id, 0, || false);
    assert_eq!(claimed_amount, RustBigUint::from(5000u32));

    fund_module_check::check_consistency(&fund_module, 3);
    assert_eq!(
        RustBigUint::from(5000u32),
        fund_module.query_sum_funds_by_type(FundType::WithdrawOnly, |_, _| true)
    );
    assert_eq!(
        1,
        fund_module.count_fund_items_by_type(FundType::WithdrawOnly, |_, _| true)
    );
    assert_eq!(
        RustBigUint::from(5000u32),
        fund_module.query_sum_funds_by_user_type(user_id, FundType::WithdrawOnly, |_| true)
    );
    assert_eq!(
        1,
        fund_module.count_fund_items_by_user_type(user_id, FundType::WithdrawOnly, |_| true)
    );

    // WithdrawOnly -> liquidate
    let liquidated = transf_module.liquidate_all_withdraw_only(user_id, || false);
    assert_eq!(liquidated, RustBigUint::from(5000u32));

    fund_module_check::check_consistency(&fund_module, 3);

    assert_eq!(
        RustBigUint::from(0u32),
        fund_module.query_sum_funds_by_type(FundType::WithdrawOnly, |_, _| true)
    );
    assert_eq!(
        0,
        fund_module.count_fund_items_by_type(FundType::WithdrawOnly, |_, _| true)
    );
    assert_eq!(
        RustBigUint::from(0u32),
        fund_module.query_sum_funds_by_user_type(user_id, FundType::WithdrawOnly, |_| true)
    );
    assert_eq!(
        0,
        fund_module.count_fund_items_by_user_type(user_id, FundType::WithdrawOnly, |_| true)
    );
}
