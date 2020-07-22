
use crate::auction_proxy::Auction;

use crate::bls_key::*;
use crate::stake_state::*;
use crate::util::*;

use crate::events::*;
use crate::nodes::*;
use crate::rewards::*;
use crate::settings::*;
use crate::user_data::*;

imports!();

#[elrond_wasm_derive::module(ContractStakeModuleImpl)]
pub trait ContractStakeModule {

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    #[module(EventsModuleImpl)]
    fn events(&self) -> EventsModuleImpl<T, BigInt, BigUint>;

    #[module(NodeModuleImpl)]
    fn nodes(&self) -> NodeModuleImpl<T, BigInt, BigUint>;

    #[module(RewardsModuleImpl)]
    fn rewards(&self) -> RewardsModuleImpl<T, BigInt, BigUint>;


    /// Yields how much stake was added to the contract.
    #[view]
    #[storage_get("filled_stake")]
    fn getFilledStake(&self) -> BigUint;

    #[private]
    #[storage_set("filled_stake")]
    fn _set_filled_stake(&self, filled_stake: &BigUint);

    #[private]
    fn _check_entire_stake_filled(&self) -> Result<(), &'static str> {
        let expected_stake = self.nodes().getExpectedStake();
        if expected_stake == 0 {
            return Err("cannot activate with 0 stake");
        }

        let filled_stake = self.getFilledStake();
        match filled_stake.cmp(&expected_stake) {
            core::cmp::Ordering::Less => {
                Err("cannot activate before all stake has been filled")
            },
            core::cmp::Ordering::Greater => {
                Err("too much stake filled")
            },
            core::cmp::Ordering::Equal => Ok(())
        }
    }

    #[view]
    #[storage_get("stake_state")]
    fn stakeState(&self) -> StakeState;

    #[private]
    #[storage_set("stake_state")]
    fn _set_stake_state(&self, active: StakeState);

    /// This is stake that is in the contract, not sent to the auction contract.
    #[private]
    #[storage_get("inactive_stake")]
    fn _get_inactive_stake(&self) -> BigUint;

    #[private]
    #[storage_set("inactive_stake")]
    fn _set_inactive_stake(&self, inactive_stake: &BigUint);


    /// Send stake to the staking contract, if the entire stake has been gathered.
    fn activate(&self,
            #[multi(self.nodes().getNumNodes())] bls_signatures: Vec<Vec<u8>>)
        -> Result<(), &str> {

        if self.get_caller() != self.settings().getContractOwner() {
            return Err("only owner can activate"); 
        }

        if !self.stakeState().is_open() {
            return Err("contract already active"); 
        }

        if self.nodes().getNumBlsKeys() != self.nodes().getNumNodes() {
            return Err("wrong number of BLS keys"); 
        }

        // check signature lengths
        for (_, signature) in bls_signatures.iter().enumerate() {
            if signature.len() != BLS_SIGNATURE_BYTE_LENGTH {
                return Err("wrong size BLS signature");
            }
        }

        let bls_keys = self.nodes().getBlsKeys();
        let num_nodes = bls_keys.len();
        if num_nodes == 0 {
            return Err("cannot activate with no nodes");
        }

        self._check_entire_stake_filled()?;

        // change state
        self._set_stake_state(StakeState::PendingActivation);

        // send all stake to auction contract
        let auction_contract_addr = self.settings().getAuctionContractAddress();
        let auction_contract = contract_proxy!(self, &auction_contract_addr, Auction);
        let total_stake = self.nodes().getExpectedStake();
        auction_contract.stake(
            num_nodes,
            zip_vectors(bls_keys, bls_signatures),
            &total_stake);

        Ok(())
    }

    /// Only finalize activation if we got confirmation from the auction contract.
    /// #[callback] can only be declared in lib.rs for the moment.
    #[private]
    fn auction_stake_callback(&self, call_result: AsyncCallResult<()>) {
        match call_result {
            AsyncCallResult::Ok(()) => {
                // set to Active
                self._set_stake_state(StakeState::Active);

                // decrease non-reward balance to account for the stake that went to the auction SC
                let mut inactive_stake = self._get_inactive_stake();
                inactive_stake -= self.nodes().getExpectedStake();
                self._set_inactive_stake(&inactive_stake);

                // log event (no data)
                self.events().activation_ok_event(());
            },
            AsyncCallResult::Err(error) => {
                // revert stake state flag
                self._set_stake_state(StakeState::OpenForStaking);

                // log failure event (no data)
                self.events().activation_fail_event(error.err_msg);
            }
        }
    }


    // DEACTIVATE + FORCE UNSTAKE

    /// Unstakes from the auction smart contract.
    /// The contract will stop receiving rewards, but stake cannot be yet reclaimed.
    /// This operation is performed by the owner.
    fn deactivate(&self) -> Result<(), &str> {
        if self.get_caller() != self.settings().getContractOwner() {
            return Err("only owner can deactivate"); 
        }

        if self.stakeState() != StakeState::Active {
            return Err("contract is not active"); 
        }

        self._perform_deactivate()
    }

    /// Delegators can force the entire contract to unstake
    /// if they put up stake for sale and no-one has bought it for long enough.
    /// This operation can be performed by any delegator.
    fn forceUnstake(&self) -> Result<(), &str> {
        let user_id = self.user_data().getUserId(&self.get_caller());
        if user_id == 0 {
            return Err("only delegators can call forceUnstake");
        }

        if self.user_data()._get_user_stake_for_sale(user_id) == 0 {
            return Err("only delegators that are trying to sell stake can call forceUnstake");
        }

        let time_of_stake_offer = self.user_data()._get_user_time_of_stake_offer(user_id);
        let time_before_force_unstake = self.settings().getTimeBeforeForceUnstake();
        if self.get_block_timestamp() <= time_of_stake_offer + time_before_force_unstake {
            return Err("too soon to call forceUnstake");
        }


 
        self._perform_deactivate()
    }

    #[private]
    fn _perform_deactivate(&self) -> Result<(), &str> {
        // change state
        self._set_stake_state(StakeState::PendingDectivation);
        
        // send unstake command to Auction SC
        let auction_contract_addr = self.settings().getAuctionContractAddress();
        let auction_contract = contract_proxy!(self, &auction_contract_addr, Auction);
        auction_contract.unStake(self.nodes().getBlsKeys());

        Ok(())
    }

    /// Only finalize deactivation if we got confirmation from the auction contract.
    /// #[callback] can only be declared in lib.rs for the moment.
    #[private]
    fn auction_unStake_callback(&self, call_result: AsyncCallResult<()>) {
        match call_result {
            AsyncCallResult::Ok(()) => {
                // set to Active
                self._set_stake_state(StakeState::UnBondPeriod);

                // log event (no data)
                self.events().deactivation_ok_event(());
            },
            AsyncCallResult::Err(error) => {
                // revert stake state flag
                self._set_stake_state(StakeState::Active);

                // log failutradere event (no data)
                self.events().deactivation_fail_event(error.err_msg);
            }
        }
    }

    // UNBOND

    /// Claims unstaked stake from the auction smart contract.
    /// This operation can be executed by anyone (note that it might cost much gas).
    fn unBond(&self) -> Result<(), &str> {
        if self.stakeState() != StakeState::UnBondPeriod {
            return Err("contract is not in unbond period"); 
        }

        let bls_keys = self.nodes().getBlsKeys();

        // save stake state flag, true
        self._set_stake_state(StakeState::PendingUnBond);

        // All rewards need to be recalculated now,
        // because after unbond the total stake can change,
        // making it impossible to correctly distribute rewards from before it changed.
        // Now performed in the callback, because gas might be insufficient there.
        self.rewards().computeAllRewards();
        
        // send unbond command to Auction SC
        let auction_contract_addr = self.settings().getAuctionContractAddress();
        let auction_contract = contract_proxy!(self, &auction_contract_addr, Auction);
        auction_contract.unBond(bls_keys);

        Ok(())
    }

    /// Only finalize deactivation if we got confirmation from the auction contract.
    /// #[callback] can only be declared in lib.rs for the moment.
    #[private]
    fn auction_unBond_callback(&self, call_result: AsyncCallResult<()>) {
        match call_result {
            AsyncCallResult::Ok(()) => {
                // open up staking
                self._set_stake_state(StakeState::OpenForStaking);

                // increase non-reward balance to account for the stake that came from the auction SC
                let mut inactive_stake = self._get_inactive_stake();
                inactive_stake += self.nodes().getExpectedStake();
                self._set_inactive_stake(&inactive_stake);

                // log event (no data)
                self.events().unBond_ok_event(());
            },
            AsyncCallResult::Err(error) => {
                // revert stake state flag
                self._set_stake_state(StakeState::UnBondPeriod);

                // log failutradere event (no data)
                self.events().unBond_fail_event(error.err_msg);
            }
        }
    }

}
