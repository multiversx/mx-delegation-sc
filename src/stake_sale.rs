use crate::user_stake_state::*;

use crate::events::*;
use crate::pause::*;
use crate::rewards::*;
use crate::user_data::*;

imports!();

/// Deals with stake trade among delegators.
/// Note: each 1 staked ERD can only be traded for 1 unstaked ERD.
#[elrond_wasm_derive::module(StakeSaleModuleImpl)]
pub trait StakeSaleModule {

    #[module(EventsModuleImpl)]
    fn events(&self) -> EventsModuleImpl<T, BigInt, BigUint>;

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(PauseModuleImpl)]
    fn pause(&self) -> PauseModuleImpl<T, BigInt, BigUint>;

    #[module(RewardsModuleImpl)]
    fn rewards(&self) -> RewardsModuleImpl<T, BigInt, BigUint>;

    /// Creates a stake offer. Overwrites any previous stake offer.
    /// Once a stake offer is up, it can be bought by anyone on a first come first served basis.
    /// Cannot be paused, because this is also part of the unStake mechanism, which the owner cannot veto.
    #[endpoint(announceUnStake)]
    fn announce_unstake(&self, amount: BigUint) -> Result<(), SCError> {
        let caller = self.get_caller();
        let user_id = self.user_data().get_user_id(&caller);
        if user_id == 0 {
            return sc_error!("only delegators can offer stake for sale")
        }

        // get active stake
        let stake = self.user_data().get_user_stake_of_type(user_id, UserStakeState::Active);
        if &amount > &stake {
            return sc_error!("cannot offer more than the user active stake")
        }

        // store offer
        self.user_data().set_user_stake_for_sale(user_id, &amount);
        self.user_data().set_user_bl_nonce_of_stake_offer(user_id, self.get_block_nonce());

        Ok(())
    }

    /// Check if a user is willing to sell stake, and how much.
    #[view(getStakeForSale)]
    fn get_stake_for_sale(&self, user: Address) -> BigUint {
        let user_id = self.user_data().get_user_id(&user);
        if user_id == 0 {
            return BigUint::zero()
        }
        self.user_data().get_user_stake_for_sale(user_id)
    }

    #[view(getTimeOfStakeOffer)]
    fn get_time_of_stake_offer(&self, user: Address) -> u64 {
        let user_id = self.user_data().get_user_id(&user);
        if user_id == 0 {
            return 0
        }
        self.user_data().get_user_bl_nonce_of_stake_offer(user_id)
    }

    /// User-to-user purchase of stake.
    /// Only stake that has been offered for sale by owner can be bought.
    /// Note: the price of 1 staked ERD must always be 1 "free" ERD, from outside the contract.
    /// The payment for the stake does not stay in the contract, it gets forwarded immediately to the seller.
    #[payable]
    #[endpoint(purchaseStake)]
    fn purchase_stake(&self, seller: Address, #[payment] payment: BigUint) -> Result<(), SCError> {
        if self.pause().is_stake_sale_paused() {
            return sc_error!("stake sale paused");
        }

        if payment == 0 {
            return Ok(())
        }

        // get seller id
        let seller_id = self.user_data().get_user_id(&seller);
        if seller_id == 0 {
            return sc_error!("unknown seller")
        }

        // decrease stake for sale
        self.user_data().update_user_stake_for_sale(seller_id, |stake_for_sale| {
            if &payment > &*stake_for_sale {
                sc_error!("payment exceeds stake offered")
            } else {
                *stake_for_sale -= &payment;
                Ok(())
            }
        })?;

        // get buyer id or create buyer
        let caller = self.get_caller();
        let mut buyer_id = self.user_data().get_user_id(&caller);
        if buyer_id == 0 {
            buyer_id = self.user_data().new_user();
            self.user_data().set_user_id(&caller, buyer_id);
        }

        // compute rewards (must happen before transferring stake):
        // for seller
        let seller_data = self.rewards().load_user_data_update_rewards(seller_id);
        self.user_data().store_user_data(seller_id, &seller_data);

        // for buyer
        let buyer_data = self.rewards().load_user_data_update_rewards(buyer_id);
        self.user_data().store_user_data(buyer_id, &buyer_data);

        // transfer stake:
        // decrease stake of seller
        let enough = self.user_data().decrease_user_stake_of_type(seller_id, UserStakeState::Active, &payment);
        if !enough {
            return sc_error!("payment exceeds seller active stake");
        }
        self.user_data().validate_total_user_stake(seller_id)?;

        // increase stake of buyer
        self.user_data().increase_user_stake_of_type(buyer_id, UserStakeState::Active, &payment);
        self.user_data().validate_total_user_stake(buyer_id)?;

        // forward payment to seller
        self.send_tx(&seller, &payment, "payment for stake");

        // log transaction
        self.events().purchase_stake_event(&seller, &caller, &payment);

        Ok(())
    }
}