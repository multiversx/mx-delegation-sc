
use user_fund_storage::fund_module::*;
use user_fund_storage::types::*;

use elrond_wasm::*;
use elrond_wasm_debug::*;

mod fund_module_check;

#[test]
fn test_fund_inc_dec_1() {
    let fund_module = FundModuleImpl::new(TxContext::dummy());
    let user_id = 2;

    fund_module.increase_fund_balance(user_id, FundDescription::Waiting, 1234u32.into());

    fund_module_check::check_consistency(&fund_module, 3);
    assert_eq!(
        RustBigUint::from(1234u32),
        fund_module.query_sum_funds_by_type(FundType::Waiting, |_, _| true));
    assert_eq!(1,
        fund_module.count_fund_items_by_type(FundType::Waiting, |_, _| true));
    assert_eq!(
        RustBigUint::from(1234u32),
        fund_module.query_sum_funds_by_user_type(user_id, FundType::Waiting, |_| true));
    assert_eq!(1,
        fund_module.count_fund_items_by_user_type(user_id, FundType::Waiting, |_| true));

    let destroyed = fund_module.destroy_all_for_user(user_id, FundType::Waiting);
    assert_eq!(destroyed, RustBigUint::from(1234u32));

    fund_module_check::check_consistency(&fund_module, 3);
    assert_eq!(
        RustBigUint::zero(),
        fund_module.query_sum_funds_by_type(FundType::Waiting, |_, _| true));
    assert_eq!(0,
        fund_module.count_fund_items_by_type(FundType::Waiting, |_, _| true));
    assert_eq!(
        RustBigUint::zero(),
        fund_module.query_sum_funds_by_user_type(user_id, FundType::Waiting, |_| true));
    assert_eq!(0,
        fund_module.count_fund_items_by_user_type(user_id, FundType::Waiting, |_| true));
}

#[test]
fn test_fund_inc_dec_2() {
    let fund_module = FundModuleImpl::new(TxContext::dummy());
    let user_id = 1;

    fund_module.increase_fund_balance(user_id, FundDescription::Waiting, 1200u32.into());
    fund_module.increase_fund_balance(user_id, FundDescription::Waiting, 34u32.into());

    fund_module_check::check_consistency(&fund_module, 3);
    assert_eq!(
        RustBigUint::from(1234u32),
        fund_module.query_sum_funds_by_type(FundType::Waiting, |_, _| true));
    assert_eq!(2,
        fund_module.count_fund_items_by_type(FundType::Waiting, |_, _| true));
    assert_eq!(
        RustBigUint::from(1234u32),
        fund_module.query_sum_funds_by_user_type(user_id, FundType::Waiting, |_| true));
    assert_eq!(2,
        fund_module.count_fund_items_by_user_type(user_id, FundType::Waiting, |_| true));

    let destroyed = fund_module.destroy_all_for_user(user_id, FundType::Waiting);
    assert_eq!(destroyed, RustBigUint::from(1234u32));

    fund_module_check::check_consistency(&fund_module, 3);
    assert_eq!(
        RustBigUint::from(0u32),
        fund_module.query_sum_funds_by_type(FundType::Waiting, |_, _| true));
    assert_eq!(0,
        fund_module.count_fund_items_by_type(FundType::Waiting, |_, _| true));
    assert_eq!(
        RustBigUint::from(0u32),
        fund_module.query_sum_funds_by_user_type(user_id, FundType::Waiting, |_| true));
    assert_eq!(0,
        fund_module.count_fund_items_by_user_type(user_id, FundType::Waiting, |_| true));

    let destroyed = fund_module.destroy_all_for_user(user_id, FundType::Waiting);
    assert_eq!(destroyed, RustBigUint::from(0u32));

    fund_module_check::check_consistency(&fund_module, 3);

    assert_eq!(
        RustBigUint::zero(),
        fund_module.query_sum_funds_by_type(FundType::Waiting, |_, _| true));
    assert_eq!(0,
        fund_module.count_fund_items_by_type(FundType::Waiting, |_, _| true));
    assert_eq!(
        RustBigUint::zero(),
        fund_module.query_sum_funds_by_user_type(user_id, FundType::Waiting, |_| true));
    assert_eq!(0,
        fund_module.count_fund_items_by_user_type(user_id, FundType::Waiting, |_| true));
}

#[test]
fn test_fund_inc_dec_3() {
    let fund_module = FundModuleImpl::new(TxContext::dummy());
    let user_id = 3;

    fund_module.increase_fund_balance(user_id, FundDescription::Waiting, 1200u32.into());
    fund_module.increase_fund_balance(user_id, FundDescription::Waiting, 34u32.into());

    fund_module_check::check_consistency(&fund_module, 3);
    assert_eq!(
        RustBigUint::from(1234u32),
        fund_module.query_sum_funds_by_type(FundType::Waiting, |_, _| true));
    assert_eq!(2,
        fund_module.count_fund_items_by_type(FundType::Waiting, |_, _| true));
    assert_eq!(
        RustBigUint::from(1234u32),
        fund_module.query_sum_funds_by_user_type(user_id, FundType::Waiting, |_| true));
    assert_eq!(2,
        fund_module.count_fund_items_by_user_type(user_id, FundType::Waiting, |_| true));

    let destroyed = fund_module.destroy_all_for_user(user_id, FundType::Waiting);
    assert_eq!(destroyed, RustBigUint::from(1234u32));

    fund_module_check::check_consistency(&fund_module, 3);
    assert_eq!(
        RustBigUint::from(0u32),
        fund_module.query_sum_funds_by_type(FundType::Waiting, |_, _| true));
    assert_eq!(0,
        fund_module.count_fund_items_by_type(FundType::Waiting, |_, _| true));
    assert_eq!(
        RustBigUint::from(0u32),
        fund_module.query_sum_funds_by_user_type(user_id, FundType::Waiting, |_| true));
    assert_eq!(0,
        fund_module.count_fund_items_by_user_type(user_id, FundType::Waiting, |_| true));
}

#[test]
fn test_transfer_funds_1() {
    let fund_module = FundModuleImpl::new(TxContext::dummy());
    let user_1 = 2;
    let user_2 = 3;

    fund_module.increase_fund_balance(user_1, FundDescription::Waiting, 1200u32.into());
    fund_module.increase_fund_balance(user_2, FundDescription::Waiting, 34u32.into());

    fund_module_check::check_consistency(&fund_module, 4);
    assert_eq!(
        RustBigUint::from(1234u32),
        fund_module.query_sum_all_funds_brute_force(|_, fund_desc| fund_desc == FundDescription::Waiting));

    let _ = fund_module.split_convert_max_by_type(
        None,
        FundType::Waiting,
        SwapDirection::Forwards,
        |_, _| Some(FundDescription::Active),
        || false,
        false,
    );

    fund_module_check::check_consistency(&fund_module, 4);
    assert_eq!(
        RustBigUint::from(1234u32),
        fund_module.query_sum_all_funds_brute_force(|_, fund_desc| fund_desc == FundDescription::Active));

    assert_eq!(
        RustBigUint::from(1234u32),
        fund_module.query_sum_funds_by_type(FundType::Active, |_, _| true));
    assert_eq!(2,
        fund_module.count_fund_items_by_type(FundType::Active, |_, _| true));

    assert_eq!(
        RustBigUint::from(1200u32),
        fund_module.query_sum_funds_by_user_type(user_1, FundType::Active, |_| true));
    assert_eq!(1,
        fund_module.count_fund_items_by_user_type(user_1, FundType::Active, |_| true));

    assert_eq!(
        RustBigUint::from(34u32),
        fund_module.query_sum_funds_by_user_type(user_2, FundType::Active, |_| true));
    assert_eq!(1,
        fund_module.count_fund_items_by_user_type(user_2, FundType::Active, |_| true));
}

#[test]
fn test_transfer_funds_2() {
    let fund_module = FundModuleImpl::new(TxContext::dummy());
    let user_1 = 2;
    let user_2 = 3;
    let user_3 = 5;

    fund_module.increase_fund_balance(user_1, FundDescription::Waiting, 1200u32.into());
    fund_module.increase_fund_balance(user_2, FundDescription::Waiting, 34u32.into());
    fund_module.increase_fund_balance(user_3, FundDescription::Waiting, 11u32.into());

    let mut amount = RustBigUint::from(1000u32);
    let affected_users = fund_module.split_convert_max_by_type(
        Some(&mut amount),
        FundType::Waiting,
        SwapDirection::Forwards,
        |_, _| Some(FundDescription::Active),
        || false,
        false,
    );

    assert_eq!(affected_users, vec![user_1]);
    assert_eq!(amount, RustBigUint::zero());

    fund_module_check::check_consistency(&fund_module, 5);
    assert_eq!(
        RustBigUint::from(1000u32),
        fund_module.query_sum_all_funds_brute_force(|_, fund_desc| fund_desc == FundDescription::Active));
    assert_eq!(
        RustBigUint::from(245u32),
        fund_module.query_sum_all_funds_brute_force(|_, fund_desc| fund_desc == FundDescription::Waiting));
}

// Going backwards
#[test]
fn test_transfer_funds_3_backwards() {
    let fund_module = FundModuleImpl::new(TxContext::dummy());
    let user_1 = 2;
    let user_2 = 3;
    let user_3 = 5;

    fund_module.increase_fund_balance(user_1, FundDescription::Waiting, 1200u32.into());
    fund_module.increase_fund_balance(user_2, FundDescription::Waiting, 34u32.into());
    fund_module.increase_fund_balance(user_3, FundDescription::Waiting, 11u32.into());

    let mut affected_users: Vec<usize> = Vec::new();
    let mut amount = RustBigUint::from(40u32);
    let returned_affected_users = fund_module.split_convert_max_by_type(
        Some(&mut amount),
        FundType::Waiting,
        SwapDirection::Backwards,
        |user_id, _| {
            affected_users.push(user_id);
            Some(FundDescription::Active)
        },
        || false,
        false,
    );

    assert_eq!(returned_affected_users, vec![user_2, user_3]);
    affected_users.sort();
    assert_eq!(returned_affected_users, affected_users);
    assert_eq!(amount, RustBigUint::zero());

    fund_module_check::check_consistency(&fund_module, 5);
    assert_eq!(
        RustBigUint::from(40u32),
        fund_module.query_sum_all_funds_brute_force(|_, fund_desc| fund_desc == FundDescription::Active));
    assert_eq!(
        RustBigUint::from(1205u32),
        fund_module.query_sum_all_funds_brute_force(|_, fund_desc| fund_desc == FundDescription::Waiting));
}

// Dry run.
#[test]
fn test_transfer_funds_4_dry_run() {
    let fund_module = FundModuleImpl::new(TxContext::dummy());
    let user_1 = 5;
    let user_2 = 7;
    let user_3 = 9;

    fund_module.increase_fund_balance(user_1, FundDescription::Waiting, 1200u32.into());
    fund_module.increase_fund_balance(user_2, FundDescription::Waiting, 34u32.into());
    fund_module.increase_fund_balance(user_3, FundDescription::Waiting, 11u32.into());

    let mut affected_users: Vec<usize> = Vec::new();
    let mut amount = RustBigUint::from(40u32);
    let returned_affected_users = fund_module.split_convert_max_by_type(
        Some(&mut amount),
        FundType::Waiting,
        SwapDirection::Backwards,
        |user_id, _| {
            affected_users.push(user_id);
            Some(FundDescription::Active)
        },
        || false,
        true,
    );

    assert_eq!(affected_users, vec![user_3, user_2]);
    affected_users.sort();
    assert_eq!(returned_affected_users, affected_users);
    assert_eq!(amount, RustBigUint::zero());

    assert_eq!(
        RustBigUint::zero(),
        fund_module.query_sum_all_funds_brute_force(|_, fund_desc| fund_desc == FundDescription::Active));
    assert_eq!(
        RustBigUint::from(1245u32),
        fund_module.query_sum_all_funds_brute_force(|_, fund_desc| fund_desc == FundDescription::Waiting));
}

#[test]
fn test_transfer_funds_5_coalesce() {
    let fund_module = FundModuleImpl::new(TxContext::dummy());
    let user_1 = 2;

    fund_module.increase_fund_balance(user_1, FundDescription::Waiting, 1000u32.into());
    fund_module.increase_fund_balance(user_1, FundDescription::Waiting, 1000u32.into());

    let mut amount = RustBigUint::from(2000u32);
    let affected_users = fund_module.split_convert_max_by_type(
        Some(&mut amount),
        FundType::Waiting,
        SwapDirection::Forwards,
        |_, _| Some(FundDescription::WithdrawOnly),
        || false,
        false,
    );

    assert_eq!(affected_users, vec![user_1]);
    assert_eq!(amount, RustBigUint::zero());

    fund_module_check::check_consistency(&fund_module, 5);
    assert_eq!(
        RustBigUint::from(2000u32),
        fund_module.query_sum_all_funds_brute_force(|_, fund_desc| fund_desc == FundDescription::WithdrawOnly));
    assert_eq!(
        RustBigUint::zero(),
        fund_module.query_sum_all_funds_brute_force(|_, fund_desc| fund_desc == FundDescription::Waiting));
}
