
use crate::stake_state::*;

use crate::events::*;
use crate::stake::*;

#[elrond_wasm_derive::module(GenesisModuleImpl)]
pub trait GenesisModule {

    #[module(EventsModuleImpl)]
    fn events(&self) -> EventsModuleImpl<T, BigInt, BigUint>;

    #[module(UserStakeModuleImpl)]
    fn stake(&self) -> UserStakeModuleImpl<T, BigInt, BigUint>;


    /// Function to be used only during genesis block.
    /// Cannot perform payments during genesis block, so we update state but not the balance.
    fn stakeGenesis(&self, stake: BigUint) -> Result<(), &str> {
        if self.get_block_nonce() > 0 {
            return Err("genesis block only")
        }
        self.stake()._process_stake(stake)
    }

    /// Function to be used only once, during genesis block.
    /// Cannot perform payments during genesis block, so we update state but do not receive or send funds.
    fn activateGenesis(&self) -> Result<(), &str> {
        if self.get_block_nonce() > 0 {
            return Err("genesis block only")
        }

        self.stake()._check_entire_stake_filled()?;

        // change state, jump directly to Active
        self.stake()._set_stake_state(StakeState::Active);

        // log event (no data)
        self.events().activation_ok_event(());

        Ok(())
    }

}