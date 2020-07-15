imports!();

use super::fund_list::*;
use super::fund_item::*;
use super::fund_type::*;


/// Deals with storage data about delegators.
#[elrond_wasm_derive::module(FundDataModuleImpl)]
pub trait UserDataModule {

    #[view(fundList)]
    #[storage_get("f")]
    fn get_fund_list(&self, discriminant: u8) -> FundList<BigUint>;

    #[storage_get_mut("f")]
    fn get_mut_fund_list(&self, discriminant: u8) -> mut_storage!(FundList<BigUint>);

    #[storage_set("f")]
    fn set_fund_list(&self, discriminant: u8, fud_list: FundList<BigUint>);

    fn split_convert_fund_item<F, C>(&self,
        max_amount: &mut BigUint, 
        source_discriminant: u8,
        filter_transform: F) -> SCResult<()> 
    where 
        F: Fn(&FundInfo) -> Option<FundInfo>,
    {
        let source_list = self.get_mut_fund_list(source_discriminant);
        // while let Some(transformed) = filter_transform()
        for fund_item in source_list.0.iter_mut() {
            
        }

        Ok(())
    }
}
