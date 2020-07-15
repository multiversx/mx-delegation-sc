use elrond_wasm::elrond_codec::*;
use elrond_wasm::Vec;
use elrond_wasm::Queue;
use elrond_wasm::BigUintApi;
use elrond_wasm::SCResult;

use super::fund_type::*;
use super::fund_item::*;
use super::fund_list::*;


// fn convert_fund_item<'a, BigUint, S, F, C>(storage_provider: S, filter: F, transformation: C) -> SCResult<()> 
// where
//     BigUint: BigUintApi + 'a,
//     S: Fn(u8) -> &'a mut FundList<BigUint>,
//     F: Fn(&FundInfo) -> bool,
//     C: Fn(&FundInfo) -> SCResult<FundInfo>,
// {
    

//     Ok(())
// }

pub fn purchase_stake(input: &FundInfo) -> Option<FundInfo> {
    if let FundType::ActiveForSale{ .. } = input.fund_type {
        
    } else {
        None
    }
}
