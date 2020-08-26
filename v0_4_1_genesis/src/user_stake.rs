
use crate::events::*;
use user_fund_storage_genesis::user_data::*;
use user_fund_storage_genesis::fund_transf_module::*;

imports!();

/// Contains endpoints for staking/withdrawing stake.
#[elrond_wasm_derive::module(UserStakeModuleImpl)]
pub trait UserStakeModule {

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(FundTransformationsModuleImpl)]
    fn fund_transf_module(&self) -> FundTransformationsModuleImpl<T, BigInt, BigUint>;

    #[module(EventsModuleImpl)]
    fn events(&self) -> EventsModuleImpl<T, BigInt, BigUint>;

    fn process_stake(&self, payment: BigUint) -> SCResult<()> {
        // get user id or create user
        // we use user id as an intermediate identifier between user address and data,
        // because we might at some point need to iterate over all user data
        let caller = self.get_caller();
        let mut user_id = self.user_data().get_user_id(&caller);
        if user_id == 0 {
            user_id = self.user_data().new_user();
            self.user_data().set_user_id(&caller, user_id);
        }

        // log staking event
        self.events().stake_event(&caller, &payment);

        // create stake funds
        self.fund_transf_module().create_waiting(user_id, payment);

        Ok(())
    }

}
