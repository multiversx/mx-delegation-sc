use crate::fund_type::*;

use crate::events::*;
use crate::pause::*;
use crate::rewards::*;
use crate::settings::*;
use crate::user_data::*;
use crate::fund_transf_module::*;
use crate::fund_view_module::*;

imports!();

/// Deals with stake trade among delegators.
/// Note: each 1 staked ERD can only be traded for 1 unstaked ERD.
#[elrond_wasm_derive::module(StakeSaleModuleImpl)]
pub trait StakeSaleModule {

    #[module(EventsModuleImpl)]
    fn events(&self) -> EventsModuleImpl<T, BigInt, BigUint>;

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(FundTransformationsModuleImpl)]
    fn fund_transf_module(&self) -> FundTransformationsModuleImpl<T, BigInt, BigUint>;

    #[module(FundViewModuleImpl)]
    fn fund_view_module(&self) -> FundViewModuleImpl<T, BigInt, BigUint>;

    #[module(PauseModuleImpl)]
    fn pause(&self) -> PauseModuleImpl<T, BigInt, BigUint>;

    #[module(RewardsModuleImpl)]
    fn rewards(&self) -> RewardsModuleImpl<T, BigInt, BigUint>;

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    /// Creates a stake offer. Overwrites any previous stake offer.
    /// Once a stake offer is up, it can be bought by anyone on a first come first served basis.
    /// Cannot be paused, because this is also part of the unStake mechanism, which the owner cannot veto.
    #[endpoint(announceUnStake)]
    fn announce_unstake(&self, amount: BigUint) -> SCResult<()> {
        if !self.settings().is_unstake_enabled() {
            return sc_error!("unstake is currently disabled");
        }
        
        let caller = self.get_caller();
        let user_id = self.user_data().get_user_id(&caller);
        if user_id == 0 {
            return sc_error!("only delegators can offer stake for sale")
        }

        // compute rewards 
        self.rewards().update_user_rewards(user_id); // for user
        self.rewards().update_user_rewards(OWNER_USER_ID); // for owner, since UnStaked will change

        // convert user stake from Active to UnStaked
        let stake = self.fund_view_module().get_user_stake_of_type(user_id, FundType::Active);
        if amount > stake {
            return sc_error!("cannot offer more than the user active stake")
        }

        // convert stake
        sc_try!(self.fund_transf_module().unstake_transf(user_id, &amount));

        Ok(())
    }

    /// Check if a user is willing to sell stake, and how much.
    #[view(getStakeForSale)]
    fn get_stake_for_sale(&self, user: Address) -> BigUint {
        let user_id = self.user_data().get_user_id(&user);
        if user_id == 0 {
            return BigUint::zero()
        }
        self.fund_view_module().get_user_stake_of_type(user_id, FundType::UnStaked)
    }

    /// User-to-user purchase of stake.
    /// Only stake that has been offered for sale by owner can be bought.
    /// Note: the price of 1 staked ERD must always be 1 "free" ERD, from outside the contract.
    #[payable]
    #[endpoint(purchaseStake)]
    fn purchase_stake(&self, seller: Address, #[payment] payment: BigUint) -> SCResult<()> {
        if self.pause().is_stake_sale_paused() {
            return sc_error!("stake sale paused");
        }
        
        let caller = self.get_caller();
        if caller == seller {
            return sc_error!("cannot purchase from self");
        }

        if payment == 0 {
            return Ok(())
        }

        // get seller id
        let seller_id = self.user_data().get_user_id(&seller);
        if seller_id == 0 {
            return sc_error!("unknown seller")
        }

        // get buyer id or create buyer
        let mut buyer_id = self.user_data().get_user_id(&caller);
        if buyer_id == 0 {
            buyer_id = self.user_data().new_user();
            self.user_data().set_user_id(&caller, buyer_id);
        }

        // compute rewards (must happen before transferring stake):
        self.rewards().update_user_rewards(seller_id); // for seller
        self.rewards().update_user_rewards(buyer_id); // for buyer
        self.rewards().update_user_rewards(OWNER_USER_ID); // for owner, since UnStaked will change

        // 1. payment from buyer becomes free stake
        self.fund_transf_module().create_free_stake(buyer_id, &payment);

        // 2a. transfer stake seller -> buyer
        // 2b. create deferred payment
        sc_try!(self.fund_transf_module().stake_sale_transf(buyer_id, seller_id, &payment));

        // log transaction
        self.events().purchase_stake_event(&seller, &caller, &payment);

        Ok(())
    }

    #[endpoint(claimPayment)]
    fn claim_payment(&self) -> SCResult<()> {
        let caller = self.get_caller();
        let caller_id = self.user_data().get_user_id(&caller);
        if caller_id == 0 {
            return sc_error!("unknown caller");
        }

        let n_blocks_before_claim =
            self.settings().get_n_blocks_before_force_unstake() +
            self.settings().get_n_blocks_before_unbond();
        let claimed_payments = sc_try!(self.fund_transf_module().claim_all_eligible_deferred_payments(
            caller_id,
            n_blocks_before_claim
        ));

        if claimed_payments > 0 {
            // forward payment to seller
            self.send_tx(&caller, &claimed_payments, "payment for stake");
        }

        Ok(())
    }
}
