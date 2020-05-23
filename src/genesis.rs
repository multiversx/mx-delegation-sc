
use crate::stake_state::*;

use crate::events::*;
use crate::stake_per_contract::*;
use crate::stake_per_user::*;

#[elrond_wasm_derive::module(GenesisModuleImpl)]
pub trait GenesisModule {

    #[module(EventsModuleImpl)]
    fn events(&self) -> EventsModuleImpl<T, BigInt, BigUint>;

    #[module(UserStakeModuleImpl)]
    fn user_stake(&self) -> UserStakeModuleImpl<T, BigInt, BigUint>;

    #[module(ContractStakeModuleImpl)]
    fn contract_stake(&self) -> ContractStakeModuleImpl<T, BigInt, BigUint>;


    /// Function to be used only during genesis block.
    /// Cannot perform payments during genesis block, so we update state but not the balance.
    fn stakeGenesis(&self, stake: BigUint) -> Result<(), &str> {
        if self.get_block_nonce() > 0 {
            return Err("genesis block only")
        }
        self.user_stake()._process_stake(stake)
    }

    /// Function to be used only once, during genesis block.
    /// Cannot perform payments during genesis block, so we update state but do not receive or send funds.
    fn activateGenesis(&self) -> Result<(), &str> {
        if self.get_block_nonce() > 0 {
            return Err("genesis block only")
        }

        self.contract_stake()._check_entire_stake_filled()?;

        // change state, jump directly to Active
        self.contract_stake()._set_stake_state(StakeState::Active);

        // log event (no data)
        self.events().activation_ok_event(());

        Ok(())
    }

}