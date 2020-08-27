imports!();

use crate::types::fund_item::*;
use crate::types::fund_type::*;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum SwapDirection {
    Forwards,
    Backwards
}

/// Deals with storage data about delegators.
#[elrond_wasm_derive::module(FundModuleImpl)]
pub trait FundModule {

    #[view(fundById)]
    #[storage_get("f")]
    fn get_fund_by_id(&self, id: usize) -> FundItem<BigUint>;

    #[storage_get_mut("f")]
    fn get_mut_fund_by_id(&self, id: usize) -> mut_storage!(FundItem<BigUint>);

    #[storage_set("f")]
    fn set_fund_by_id(&self, id: usize, fund_item: &FundItem<BigUint>);

    #[storage_get("f_max_id")]
    fn get_fund_max_id(&self) -> usize;

    #[storage_set("f_max_id")]
    fn set_fund_max_id(&self, f_num: usize);

    #[storage_get("ftype")]
    fn get_fund_list_by_type(&self, fund_type: FundType) -> FundsListInfo<BigUint>;

    #[storage_get_mut("ftype")]
    fn get_mut_fund_list_by_type(&self, fund_type: FundType) -> mut_storage!(FundsListInfo<BigUint>);

    #[storage_get("fuser")]
    fn get_fund_list_by_user(&self, user_id: usize, fund_type: FundType) -> FundsListInfo<BigUint>;

    #[storage_get_mut("fuser")]
    fn get_mut_fund_list_by_user(&self, user_id: usize, fund_type: FundType) -> mut_storage!(FundsListInfo<BigUint>);

    /// For testing; please do not use in production.
    /// Goes through all fund items, ignores indexes.
    fn query_sum_all_funds_brute_force<F>(&self, filter: F) -> BigUint 
    where 
        F: Fn(usize, FundDescription) -> bool,
    {
        let mut sum = BigUint::zero();
        let max_fund_id = self.get_fund_max_id();
        for id in 1..(max_fund_id+1) {
            let fund_item = self.get_fund_by_id(id);
            if filter(fund_item.user_id, fund_item.fund_desc) {
                sum += &fund_item.balance;
            }
        }
        sum
    }

    fn query_sum_funds_by_type<F>(&self, fund_type: FundType, filter: F) -> BigUint 
    where 
        F: Fn(usize, FundDescription) -> bool,
    {
        let mut sum = BigUint::zero();
        let type_list = self.get_fund_list_by_type(fund_type);
        let mut id = type_list.first;
        while id > 0 {
            let fund_item = self.get_fund_by_id(id);
            if filter(fund_item.user_id, fund_item.fund_desc) {
                sum += &fund_item.balance;
            }
            id = fund_item.type_list_next;
        }
        sum
    }

    fn query_sum_funds_by_user_type<F>(&self, user_id: usize, fund_type: FundType, filter: F) -> BigUint 
    where 
        F: Fn(FundDescription) -> bool,
    {
        let mut sum = BigUint::zero();
        let user_list = self.get_fund_list_by_user(user_id, fund_type);
        let mut id = user_list.first;
        while id > 0 {
            let fund_item = self.get_fund_by_id(id);
            if filter(fund_item.fund_desc) {
                sum += &fund_item.balance;
            }
            id = fund_item.user_list_next;
        }
        sum
    }

    /// Mostly written for testing. The contract shouldn't care how many items there are in the list.
    fn count_fund_items_by_type<F>(&self, fund_type: FundType, filter: F) -> usize 
    where 
        F: Fn(usize, FundDescription) -> bool,
    {
        let mut count = 0usize;
        let type_list = self.get_fund_list_by_type(fund_type);
        let mut id = type_list.first;
        while id > 0 {
            let fund_item = self.get_fund_by_id(id);
            if filter(fund_item.user_id, fund_item.fund_desc) {
                count += 1;
            }
            id = fund_item.type_list_next;
        }
        count
    }

    /// Mostly written for testing. The contract shouldn't care how many items there are in the list.
    fn count_fund_items_by_user_type<F>(&self, user_id: usize, fund_type: FundType, filter: F) -> usize 
    where 
        F: Fn(FundDescription) -> bool,
    {
        let mut count = 0usize;
        let user_list = self.get_fund_list_by_user(user_id, fund_type);
        let mut id = user_list.first;
        while id > 0 {
            let fund_item = self.get_fund_by_id(id);
            if filter(fund_item.fund_desc) {
                count += 1;
            }
            id = fund_item.user_list_next;
        }
        count
    }

    /// Adds at the end of the fund by type list.
    fn add_fund_to_type_list(&self, id: usize, new_fund_item: &mut FundItem<BigUint>) {
        let mut type_list = self.get_mut_fund_list_by_type(new_fund_item.fund_desc.fund_type());
        if type_list.is_empty() {
            type_list.first = id;
            type_list.last = id;
        } else {
            new_fund_item.type_list_prev = type_list.last;
            let mut prev_fund = self.get_mut_fund_by_id(type_list.last);
            (*prev_fund).type_list_next = id;
            type_list.last = id;
        }
        type_list.total_balance += &new_fund_item.balance;
    }

    /// Adds at the end of the fund by user+type list.
    fn add_fund_to_user_list(&self, id: usize, new_fund_item: &mut FundItem<BigUint>) {
        let mut user_list = self.get_mut_fund_list_by_user(new_fund_item.user_id, new_fund_item.fund_desc.fund_type());
        if user_list.is_empty() {
            user_list.first = id;
            user_list.last = id;
        } else {
            new_fund_item.user_list_prev = user_list.last;
            let mut prev_fund = self.get_mut_fund_by_id(user_list.last);
            (*prev_fund).user_list_next = id;
            user_list.last = id;
        }
        user_list.total_balance += &new_fund_item.balance;
    }

    fn create_fund(&self, user_id: usize, fund_desc: FundDescription, balance: BigUint) {
        // add fund
        let mut fund_max_id = self.get_fund_max_id();
        fund_max_id += 1;
        self.set_fund_max_id(fund_max_id);

        let mut new_fund_item = FundItem{
            fund_desc,
            user_id,
            balance,
            type_list_next: 0,
            type_list_prev: 0,
            user_list_next: 0,
            user_list_prev: 0,
        };

        self.add_fund_to_type_list(fund_max_id, &mut new_fund_item);
        self.add_fund_to_user_list(fund_max_id, &mut new_fund_item);

        self.set_fund_by_id(fund_max_id, &new_fund_item);
    }

    fn increase_fund_balance(&self, user_id: usize, fund_desc: FundDescription, amount: BigUint) {
        // attempt to coalesce into 1 fund item
        if fund_desc.fund_type().allow_coalesce() { // not all types can be coalesced, anything involving queues cannot
            let user_list = self.get_fund_list_by_user(user_id, fund_desc.fund_type());
            if !user_list.is_empty() { // at least 1 item must exist for user
                let mut last_item = self.get_fund_by_id(user_list.last);
                if last_item.fund_desc == fund_desc { // specific item descriptions need to be identical
                    last_item.balance += &amount;
                    self.set_fund_by_id(user_list.last, &last_item);
                    return;
                }
            }
        }

        self.create_fund(user_id, fund_desc, amount);
    }

    fn delete_fund_from_type_list(&self,
        fund_item: &mut FundItem<BigUint>,
        type_list: &mut FundsListInfo<BigUint>) {

        if fund_item.type_list_prev == 0 {
            type_list.first = fund_item.type_list_next;
        } else {
            let mut prev = self.get_mut_fund_by_id(fund_item.type_list_prev);
            (*prev).type_list_next = fund_item.type_list_next;
        }
        fund_item.type_list_prev = 0; // also clear own prev, so the item can be deleted

        if fund_item.type_list_next == 0 {
            type_list.last = fund_item.type_list_prev;
        } else {
            let mut next = self.get_mut_fund_by_id(fund_item.type_list_next);
            (*next).type_list_prev = fund_item.type_list_prev;
        }
        fund_item.type_list_next = 0; // also clear own next, so the item can be deleted
    }

    fn delete_fund_from_user_list(&self,
        fund_item: &mut FundItem<BigUint>,
        user_list: &mut FundsListInfo<BigUint>) {
    
        if fund_item.user_list_prev == 0 {
            user_list.first = fund_item.user_list_next;
        } else {
            let mut prev = self.get_mut_fund_by_id(fund_item.user_list_prev);
            (*prev).user_list_next = fund_item.user_list_next;
        }
        fund_item.user_list_prev = 0; // also clear own prev, so the item can be deleted
    
        if fund_item.user_list_next == 0 {
            user_list.last = fund_item.user_list_prev;
        } else {
            let mut next = self.get_mut_fund_by_id(fund_item.user_list_next);
            (*next).user_list_prev = fund_item.user_list_prev;
        }
        fund_item.user_list_next = 0; // also clear own next, so the item can be deleted
    }

    /// Returns the old balance of the deleted item.
    fn delete_fund(&self, fund_item: &mut FundItem<BigUint>) -> BigUint {
        let mut type_list = self.get_mut_fund_list_by_type(fund_item.fund_desc.fund_type());
        let mut user_list = self.get_mut_fund_list_by_user(fund_item.user_id, fund_item.fund_desc.fund_type());

        // synchronize sums
        (*type_list).total_balance -= &fund_item.balance;
        (*user_list).total_balance -= &fund_item.balance;

        // remove fund from the linked lists
        self.delete_fund_from_type_list(fund_item, &mut *type_list);
        self.delete_fund_from_user_list(fund_item, &mut *user_list);

        // setting balance to zero causes the fund item to be removed from storage when saving
        // result = fund_item.balance; fund_item.balance = 0;
        core::mem::replace(&mut fund_item.balance, BigUint::zero())
    }

    /// Decreases `amount` and returns by how much it decreased.
    fn decrease_fund_balance(&self, amount: &mut BigUint, fund_item: &mut FundItem<BigUint>) -> BigUint {
        let mut type_list = self.get_mut_fund_list_by_type(fund_item.fund_desc.fund_type());
        let mut user_list = self.get_mut_fund_list_by_user(fund_item.user_id, fund_item.fund_desc.fund_type());

        if *amount >= fund_item.balance {
            *amount -= &fund_item.balance;
            self.delete_fund(fund_item)
        } else {
            // consume all the remaining amount
            fund_item.balance -= &*amount;

            // synchronize sums
            (*type_list).total_balance -= &*amount;
            (*user_list).total_balance -= &*amount;

            // result = amount; amount = 0;
            core::mem::replace(amount, BigUint::zero())
        }
    }

    fn split_convert_max_by_type<F, I>(&self,
        mut opt_max_amount: Option<&mut BigUint>,
        source_type: FundType,
        direction: SwapDirection,
        mut filter_transform: F,
        interrupt: I,
        dry_run: bool)
    where 
        F: FnMut(usize, FundDescription) -> Option<FundDescription>,
        I: Fn() -> bool
    {
        let type_list = self.get_fund_list_by_type(source_type);
        let mut id = match direction {
            SwapDirection::Forwards => type_list.first,
            SwapDirection::Backwards => type_list.last,
        };

        while id > 0 && !interrupt() {
            if let Some(max_amount) = &opt_max_amount {
                if **max_amount == 0 {
                    return; // do not process anything after the max_amount is completely drained
                }
            }

            let mut fund_item = self.get_mut_fund_by_id(id);
            let next_id = match direction { // save next id now, because fund_item can be destroyed
                SwapDirection::Forwards => fund_item.type_list_next,
                SwapDirection::Backwards => fund_item.type_list_prev,
            };

            if let Some(transformed) = filter_transform(fund_item.user_id, fund_item.fund_desc) {
                if dry_run {
                    // only decrease max_amount
                    if let Some(max_amount) = opt_max_amount {
                        if fund_item.balance >= *max_amount {
                            *max_amount = BigUint::zero();
                        } else {
                            *max_amount -= &fund_item.balance;
                        }
                        opt_max_amount = Some(max_amount); // move back
                    }
                } else {
                    // extract / decrease
                    let extracted_balance: BigUint;
                    if let Some(max_amount) = opt_max_amount {
                        extracted_balance = self.decrease_fund_balance(max_amount, &mut *fund_item);
                        opt_max_amount = Some(max_amount); // move back
                    } else {
                        extracted_balance = self.delete_fund(&mut *fund_item);
                    }
                    // create / increase
                    self.increase_fund_balance(
                        (*fund_item).user_id,
                        transformed,
                        extracted_balance);
                }
            }
            id = next_id;
        }
    }

    fn split_convert_max_by_user<F>(&self,
        mut opt_max_amount: Option<&mut BigUint>,
        user_id: usize,
        source_type: FundType,
        filter_transform: F) -> SCResult<BigUint> 
    where 
        F: Fn(FundDescription) -> Option<FundDescription>,
    {
        let user_list = self.get_fund_list_by_user(user_id, source_type);
        let mut id = user_list.first;
        let mut total_transformed = BigUint::zero();

        while id > 0 {
            let mut fund_item = self.get_mut_fund_by_id(id);
            let next_id = fund_item.user_list_next; // save next id now, because fund_item can be destroyed

            if let Some(transformed) = filter_transform(fund_item.fund_desc) {
                // extract / decrease
                let extracted_balance: BigUint;
                if let Some(max_amount) = opt_max_amount {
                    extracted_balance = self.decrease_fund_balance(max_amount, &mut *fund_item);
                    opt_max_amount = Some(max_amount); // move back
                } else {
                    extracted_balance = self.delete_fund(&mut *fund_item);
                }
                // add to sum
                total_transformed += &extracted_balance;
                // create / increase
                self.increase_fund_balance(
                    (*fund_item).user_id,
                    transformed,
                    extracted_balance);
            }
            id = next_id;
        }

        Ok(total_transformed)
    }

    fn destroy_max_for_user(&self,
        amount: &mut BigUint,
        user_id: usize,
        source_type: FundType) -> SCResult<()> {

        let user_list = self.get_fund_list_by_user(user_id, source_type);
        let mut id = user_list.first;

        while id > 0 {
            let mut fund_item = self.get_mut_fund_by_id(id);
            let next_id = fund_item.user_list_next; // save next id now, because fund_item can be destroyed

            let _ = self.decrease_fund_balance(amount, &mut *fund_item);
            id = next_id;
        }

        Ok(())
    }
}
