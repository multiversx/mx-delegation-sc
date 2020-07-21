imports!();

use super::fund_list::*;
use super::fund_item::*;
use super::fund_type::*;


/// Deals with storage data about delegators.
#[elrond_wasm_derive::module(FundModuleImpl)]
pub trait FundModule {

    #[view(fundList)]
    #[storage_get("f")]
    fn get_fund_list(&self, discriminant: u8) -> FundList<BigUint>;

    #[storage_get_mut("f")]
    fn get_mut_fund_list(&self, discriminant: u8) -> mut_storage!(FundList<BigUint>);

    #[storage_set("f")]
    fn set_fund_list(&self, discriminant: u8, fund_list: FundList<BigUint>);

    fn query_list<F>(&self, discriminant: u8, filter: F) -> BigUint 
    where 
        F: Fn(&FundInfo) -> bool,
    {
        let mut sum = BigUint::zero();
        let list = self.get_fund_list(discriminant);
        for fund_item in list.0.iter() {
            if filter(&fund_item.info) {
                sum += &fund_item.balance;
            }
        }
        sum
    }

    fn query_all<F>(&self, filter: F) -> BigUint 
    where 
        F: Fn(&FundInfo) -> bool,
    {
        let mut sum = BigUint::zero();
        for discriminant in 0..10 {
            let mut list = self.get_fund_list(discriminant);
            for fund_item in list.0.iter_mut() {
                if filter(&fund_item.info) {
                    sum += &fund_item.balance;
                }
            }
        }
        sum
    }

    fn find_in_list_map<F, R>(&self, discriminant: u8, f: F) -> Option<R> 
    where 
        F: Fn(&FundInfo) -> Option<R>,
    {
        let list = self.get_fund_list(discriminant);
        for fund_item in list.0.iter() {
            if let Some(r) = f(&fund_item.info) {
                return Some(r);
            }
        }
        None
    }

    fn create_fund(&self, fund_info: FundInfo, balance: BigUint) {
        let dest_discriminant = fund_info.fund_desc.discriminant();
        let mut dest_list = self.get_mut_fund_list(dest_discriminant);
        let new_fund_item = FundItem{
            info: fund_info,
            balance,
        };
        (*dest_list).push(new_fund_item);
    }

    fn extract_balance(&self, amount: &mut BigUint, fund_item: &mut FundItem<BigUint>) -> BigUint {
        let extracted_balance: BigUint;
        if *amount > fund_item.balance {
            // consume the entire fund
            // amount <- amount - fund_item.balance
            // extracted_balance <- fund_item.balance
            // fund_item.balance <- 0
            *amount -= &fund_item.balance;
            extracted_balance = core::mem::replace(&mut fund_item.balance, BigUint::zero());
        } else {
            // consume all the remaining amount
            // amount <- 0
            // extracted_balance <- amount
            // fund_item.balance <- fund_item.balance - amount
            fund_item.balance -= &*amount;
            extracted_balance = core::mem::replace(amount, BigUint::zero());
        }
        extracted_balance
    }

    fn destroy_max<F>(&self,
        amount: &mut BigUint, 
        source_discriminant: u8,
        filter: F)
    where 
        F: Fn(&FundInfo) -> bool,
    {
        let mut source_list = self.get_mut_fund_list(source_discriminant);
        for fund_item in source_list.0.iter_mut() {
            if *amount == 0 {
                break;
            }
            if filter(&fund_item.info) {
                let _ = self.extract_balance(amount, fund_item);
            }
        }
    }



    fn split_convert_max<F>(&self,
        amount: &mut BigUint, 
        source_discriminant: u8,
        dest_discriminant: u8,
        filter_transform: F) -> SCResult<()> 
    where 
        F: Fn(&FundInfo) -> Option<FundDescription>,
    {
        let mut source_list = self.get_mut_fund_list(source_discriminant);
        let mut dest_list_opt: Option<mut_storage!(FundList<BigUint>)> = None;

        for fund_item in source_list.0.iter_mut() {
            if *amount == 0 {
                break;
            }
            if let Some(transformed) = filter_transform(&fund_item.info) {
                let extracted_balance = self.extract_balance(amount, fund_item);

                let new_fund_item = FundItem{
                    info: FundInfo {
                        user_id: fund_item.info.user_id, // user id cannot change
                        fund_desc: transformed,
                    },
                    balance: extracted_balance,
                };
                
                let mut dest_list = dest_list_opt.unwrap_or_else(|| self.get_mut_fund_list(dest_discriminant));
                (*dest_list).push(new_fund_item);
                dest_list_opt = Some(dest_list);
            }
        }

        Ok(())
    }

    fn split_convert_all<F>(&self,
        source_discriminant: u8,
        dest_discriminant: u8,
        filter_transform: F) -> SCResult<BigUint> 
    where 
        F: Fn(&FundInfo) -> Option<FundInfo>,
    {
        let mut source_list = self.get_mut_fund_list(source_discriminant);
        let mut dest_list_opt: Option<mut_storage!(FundList<BigUint>)> = None;
        let mut sum = BigUint::zero();

        for fund_item in source_list.0.iter_mut() {
            if let Some(transformed) = filter_transform(&fund_item.info) {
                sum += &fund_item.balance;
                let split_balance = core::mem::replace(&mut fund_item.balance, BigUint::zero());

                let new_fund_item = FundItem{
                    info: transformed,
                    balance: split_balance,
                };
                
                let mut dest_list = dest_list_opt.unwrap_or_else(|| self.get_mut_fund_list(dest_discriminant));
                (*dest_list).push(new_fund_item);
                dest_list_opt = Some(dest_list);
            }
        }

        Ok(sum)
    }
}
