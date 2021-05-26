use elrond_wasm::api::BigUintApi;
use elrond_wasm::elrond_codec::test_util::*;
use elrond_wasm::elrond_codec::*;
use elrond_wasm_debug::api::RustBigUint;
use user_fund_storage::types::{FundDescription, FundItem, FundType};

fn check<T: TopEncode + TopDecode + PartialEq + core::fmt::Debug>(t: T) {
    let serialized_bytes = check_top_encode(&t);
    let deserialized: T = check_top_decode::<T>(&serialized_bytes[..]);
    assert_eq!(deserialized, t);
}

#[test]
fn test_fund_type_serialization() {
    check(FundType::WithdrawOnly);
    check(FundType::Waiting);
    check(FundType::Active);
    check(FundType::UnStaked);
    check(FundType::DeferredPayment);
}

#[test]
fn test_fund_description_serialization() {
    check(FundDescription::WithdrawOnly);
    check(FundDescription::Waiting { created: 31 });
    check(FundDescription::Active);
    check(FundDescription::UnStaked { created: 5 });
    check(FundDescription::DeferredPayment { created: 20 });
}

#[test]
fn test_fund_item_empty_serialization() {
    let empty = FundItem {
        fund_desc: FundDescription::Active,
        user_id: 123,
        balance: RustBigUint::from(0u32),
        type_list_next: 0,
        type_list_prev: 0,
        user_list_next: 0,
        user_list_prev: 0,
    };

    let serialized_bytes = check_top_encode(&empty);
    assert!(serialized_bytes.is_empty());
    let deserialized: FundItem<RustBigUint> =
        check_top_decode::<FundItem<RustBigUint>>(&serialized_bytes[..]);
    assert_eq!(
        deserialized,
        FundItem {
            fund_desc: FundDescription::WithdrawOnly,
            user_id: 0,
            balance: RustBigUint::zero(),
            type_list_next: 0,
            type_list_prev: 0,
            user_list_next: 0,
            user_list_prev: 0,
        }
    );
}

#[test]
fn test_fund_item_serialization_1() {
    check(FundItem {
        fund_desc: FundDescription::Active,
        user_id: 123,
        balance: RustBigUint::from(7usize),
        type_list_next: 15,
        type_list_prev: 16,
        user_list_next: 17,
        user_list_prev: 18,
    });
}

#[test]
fn test_fund_item_serialization_2() {
    check(FundItem {
        fund_desc: FundDescription::DeferredPayment { created: 20 },
        user_id: 5,
        balance: RustBigUint::from(1usize),
        type_list_next: 10000,
        type_list_prev: 0,
        user_list_next: 0,
        user_list_prev: 3,
    });
}
