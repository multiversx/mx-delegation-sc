
// fn main() {}

use user_fund_storage::fund_module::*;
use user_fund_storage::types::*;

// use sc_delegation_rs::*;
use elrond_wasm_debug::*;
use elrond_wasm_debug::HashMap;

static ADDR: [u8; 32] = [0x11u8; 32];

fn main() {
    let mock_ref = ArwenMockState::new_ref();

    mock_ref.add_account(AccountData{
        address: ADDR.into(),
        nonce: 0,
        balance: 0.into(),
        storage: HashMap::new(),
        contract: None,
    });

    let fund_module = FundModuleImpl::new(mock_ref.clone());

    // mock_ref.call_private_contract_method(&ADDR.into(), || {
    //     fund_module.increase_fund_balance(5, FundDescription::Waiting, 1usize.into());
    // });

    mock_ref.call_private_contract_method(&ADDR.into(), || {
        fund_module.increase_fund_balance(5, FundDescription::Waiting, 0x1234usize.into());
    });

    mock_ref.print_accounts();
    
}

