use elrond_wasm::api::BigUintApi;

use super::fund_type::FundDescription;

elrond_wasm::derive_imports!();

/// A unit of balance, usually stake.
/// Contains a description of the source/intent of the funds, together with a balance.
#[derive(
    TopEncodeOrDefault, TopDecodeOrDefault, NestedEncode, NestedDecode, TypeAbi, PartialEq, Debug,
)]
pub struct FundItem<BigUint: BigUintApi> {
    pub fund_desc: FundDescription,
    pub user_id: usize,
    pub balance: BigUint,
    pub type_list_next: usize,
    pub type_list_prev: usize,
    pub user_list_next: usize,
    pub user_list_prev: usize,
}

impl<BigUint: BigUintApi> elrond_codec::EncodeDefault for FundItem<BigUint> {
    fn is_default(&self) -> bool {
        self.balance == 0
            && self.type_list_next == 0
            && self.type_list_prev == 0
            && self.user_list_next == 0
            && self.user_list_prev == 0
    }
}

impl<BigUint: BigUintApi> elrond_codec::DecodeDefault for FundItem<BigUint> {
    fn default() -> Self {
        FundItem {
            fund_desc: FundDescription::WithdrawOnly,
            user_id: 0,
            balance: BigUint::zero(),
            type_list_next: 0,
            type_list_prev: 0,
            user_list_next: 0,
            user_list_prev: 0,
        }
    }
}
