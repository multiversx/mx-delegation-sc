
use user_fund_storage::fund_module::*;
use user_fund_storage::types::*;

use elrond_wasm::*;
use elrond_wasm_debug::*;
use elrond_wasm_debug::HashMap;

static ADDR: [u8; 32] = [11u8; 32];

fn set_up_module_to_test() -> FundModuleImpl<ArwenMockRef, RustBigInt, RustBigUint> {
    let mock_ref = ArwenMockState::new_ref();
    mock_ref.add_account(AccountData{
        address: ADDR.into(),
        nonce: 0,
        balance: 0.into(),
        storage: HashMap::new(),
        contract: None,
    });
    mock_ref.set_dummy_tx(&ADDR.into());

    FundModuleImpl::new(mock_ref.clone())
}

#[test]
fn test_fund_inc_dec_1() {
    let fund_module = set_up_module_to_test();
    let user_id = 5;

    fund_module.increase_fund_balance(user_id, FundDescription::Waiting, 1234u32.into());

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

    let mut destroy = RustBigUint::from(1234u32);
    let sc_res = fund_module.destroy_max_for_user(&mut destroy, user_id, FundType::Waiting);
    assert!(sc_res.is_ok());

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
    let fund_module = set_up_module_to_test();
    let user_id = 5;

    fund_module.increase_fund_balance(user_id, FundDescription::Waiting, 1200u32.into());
    fund_module.increase_fund_balance(user_id, FundDescription::Waiting, 34u32.into());

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

    let mut destroy = RustBigUint::from(1200u32);
    let sc_res = fund_module.destroy_max_for_user(&mut destroy, user_id, FundType::Waiting);
    assert!(sc_res.is_ok());

    assert_eq!(
        RustBigUint::from(34u32),
        fund_module.query_sum_funds_by_type(FundType::Waiting, |_, _| true));
    assert_eq!(1,
        fund_module.count_fund_items_by_type(FundType::Waiting, |_, _| true));
    assert_eq!(
        RustBigUint::from(34u32),
        fund_module.query_sum_funds_by_user_type(user_id, FundType::Waiting, |_| true));
    assert_eq!(1,
        fund_module.count_fund_items_by_user_type(user_id, FundType::Waiting, |_| true));

    let mut destroy = RustBigUint::from(1000034u32);
    let sc_res = fund_module.destroy_max_for_user(&mut destroy, user_id, FundType::Waiting);
    assert!(sc_res.is_ok());

    assert_eq!(RustBigUint::from(1000000u32), destroy);

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
    let fund_module = set_up_module_to_test();
    let user_id = 5;

    fund_module.increase_fund_balance(user_id, FundDescription::Waiting, 1200u32.into());
    fund_module.increase_fund_balance(user_id, FundDescription::Waiting, 34u32.into());

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

    let mut destroy = RustBigUint::from(1230u32);
    let sc_res = fund_module.destroy_max_for_user(&mut destroy, user_id, FundType::Waiting);
    assert!(sc_res.is_ok());
    assert_eq!(destroy, RustBigUint::zero());

    assert_eq!(
        RustBigUint::from(4u32),
        fund_module.query_sum_funds_by_type(FundType::Waiting, |_, _| true));
    assert_eq!(1,
        fund_module.count_fund_items_by_type(FundType::Waiting, |_, _| true));
    assert_eq!(
        RustBigUint::from(4u32),
        fund_module.query_sum_funds_by_user_type(user_id, FundType::Waiting, |_| true));
    assert_eq!(1,
        fund_module.count_fund_items_by_user_type(user_id, FundType::Waiting, |_| true));
}

#[test]
fn test_transfer_funds_1() {
    let fund_module = set_up_module_to_test();
    let user_1 = 5;
    let user_2 = 7;

    fund_module.increase_fund_balance(user_1, FundDescription::Waiting, 1200u32.into());
    fund_module.increase_fund_balance(user_2, FundDescription::Waiting, 34u32.into());

    assert_eq!(
        RustBigUint::from(1234u32),
        fund_module.query_sum_all_funds_brute_force(|_, fund_desc| fund_desc == FundDescription::Waiting));

    let _ = fund_module.split_convert_max_by_type(
        None,
        FundType::Waiting,
        SwapDirection::Forwards,
        |_, _| Some(FundDescription::Active),
        || false,
    );

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
    let fund_module = set_up_module_to_test();
    let user_1 = 5;
    let user_2 = 7;
    let user_3 = 9;

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
    );

    assert_eq!(affected_users, vec![user_1]);
    assert_eq!(amount, RustBigUint::zero());

    assert_eq!(
        RustBigUint::from(1000u32),
        fund_module.query_sum_all_funds_brute_force(|_, fund_desc| fund_desc == FundDescription::Active));
    assert_eq!(
        RustBigUint::from(245u32),
        fund_module.query_sum_all_funds_brute_force(|_, fund_desc| fund_desc == FundDescription::Waiting));
}

// Going backwards
#[test]
fn test_transfer_funds_3() {
    let fund_module = set_up_module_to_test();
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
    );

    assert_eq!(returned_affected_users, vec![user_2, user_3]);
    affected_users.sort();
    assert_eq!(returned_affected_users, affected_users);
    assert_eq!(amount, RustBigUint::zero());

    assert_eq!(
        RustBigUint::from(40u32),
        fund_module.query_sum_all_funds_brute_force(|_, fund_desc| fund_desc == FundDescription::Active));
    assert_eq!(
        RustBigUint::from(1205u32),
        fund_module.query_sum_all_funds_brute_force(|_, fund_desc| fund_desc == FundDescription::Waiting));
}

// Dry run.
#[test]
fn test_transfer_funds_4() {
    let fund_module = set_up_module_to_test();
    let user_1 = 5;
    let user_2 = 7;
    let user_3 = 9;

    fund_module.increase_fund_balance(user_1, FundDescription::Waiting, 1200u32.into());
    fund_module.increase_fund_balance(user_2, FundDescription::Waiting, 34u32.into());
    fund_module.increase_fund_balance(user_3, FundDescription::Waiting, 11u32.into());

    let mut affected_users: Vec<usize> = Vec::new();
    let mut amount = RustBigUint::from(40u32);
    let returned_affected_users = fund_module.get_affected_users_of_swap(
        Some(&mut amount),
        FundType::Waiting,
        SwapDirection::Backwards,
        |user_id, _| {
            affected_users.push(user_id);
            true
        },
        || false,
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
