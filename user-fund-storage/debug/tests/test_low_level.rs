
// use elrond_wasm::elrond_codec::*;
// use sc_delegation_rs::bls_key::*;
// use elrond_wasm::Vec;
use user_fund_storage::fund_module::*;
use user_fund_storage::types::*;

// use sc_delegation_rs::*;
use elrond_wasm_debug::*;
use elrond_wasm_debug::HashMap;

static ADDR: [u8; 32] = [0x11u8; 32];

#[test]
fn test_increase_fund_1() {
    let mock_ref = ArwenMockState::new_ref();
    mock_ref.add_account(AccountData{
        address: ADDR.into(),
        nonce: 0,
        balance: 0.into(),
        storage: HashMap::new(),
        contract: None,
    });
    mock_ref.set_dummy_tx(&ADDR.into());

    let fund_module = FundModuleImpl::new(mock_ref.clone());

    fund_module.increase_fund_balance(5, FundDescription::Waiting, 0x1234usize.into());

    assert_eq!(
        RustBigUint::from(0x1234usize),
        fund_module.query_sum_funds_by_type(FundType::Waiting, |_, _| true));
}

#[test]
fn test_increase_fund_2() {
    let mock_ref = ArwenMockState::new_ref();
    mock_ref.add_account(AccountData{
        address: ADDR.into(),
        nonce: 0,
        balance: 0.into(),
        storage: HashMap::new(),
        contract: None,
    });
    mock_ref.set_dummy_tx(&ADDR.into());

    let fund_module = FundModuleImpl::new(mock_ref.clone());


    fund_module.increase_fund_balance(5, FundDescription::Waiting, 1usize.into());
    fund_module.increase_fund_balance(5, FundDescription::Waiting, 0x1234usize.into());

    assert_eq!(
        RustBigUint::from(0x1235usize),
        fund_module.query_sum_funds_by_type(FundType::Waiting, |_, _| true));
}
