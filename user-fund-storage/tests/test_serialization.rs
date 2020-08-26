
use elrond_wasm::elrond_codec::*;
use elrond_wasm::BigUintApi;
use elrond_wasm_debug::*;
use user_fund_storage::types::*;

fn check<T: Encode + Decode + PartialEq + std::fmt::Debug>(t: T) {
    let serialized_bytes = t.top_encode().unwrap();
    let deserialized: T = decode_from_byte_slice(&serialized_bytes[..]).unwrap();
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
    check(FundDescription::Waiting);
    check(FundDescription::Active);
    check(FundDescription::UnStaked{ created: 5 });
    check(FundDescription::DeferredPayment{ created: 20 });
}

#[test]
fn test_fund_item_empty_serialization() {
    let empty = FundItem{
        fund_desc: FundDescription::Active,
        user_id: 123,
        balance: RustBigUint::from(0u32),
        type_list_next: 0,
        type_list_prev: 0,
        user_list_next: 0,
        user_list_prev: 0,
    };

    let serialized_bytes = empty.top_encode().unwrap();
    assert!(serialized_bytes.is_empty());
    let deserialized: FundItem<RustBigUint> = decode_from_byte_slice(&serialized_bytes[..]).unwrap();
    assert_eq!(deserialized, FundItem{
        fund_desc: FundDescription::WithdrawOnly,
        user_id: 0,
        balance: RustBigUint::zero(),
        type_list_next: 0,
        type_list_prev: 0,
        user_list_next: 0,
        user_list_prev: 0,
    });
}

#[test]
fn test_fund_item_serialization_1() {
    check(FundItem{
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
    check(FundItem{
        fund_desc: FundDescription::DeferredPayment{ created: 20 },
        user_id: 5,
        balance: RustBigUint::from(1usize),
        type_list_next: 10000,
        type_list_prev: 0,
        user_list_next: 0,
        user_list_prev: 3,
    });
}
