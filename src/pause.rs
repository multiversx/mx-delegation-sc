
use crate::settings::*;

imports!();

/// The module deals with temporarily pausing certain operations.
/// 
#[elrond_wasm_derive::module(PauseModuleImpl)]
pub trait PauseModule {

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    #[view(isStakingPaused)]
    #[storage_get("staking_paused")]
    fn is_staking_paused(&self) -> bool;

    #[storage_set("staking_paused")]
    fn set_staking_paused(&self, staking_paused: bool);

    #[endpoint(pauseStaking)]
    fn pause_staking(&self) -> SCResult<()>{
        if !self.settings().owner_called() {
            return sc_error!("only owner can pause staking");
        }
        self.set_staking_paused(true);
        // TODO: event
        Ok(())
    }

    #[endpoint(unpauseStaking)]
    fn unpause_staking(&self) -> SCResult<()>{
        if !self.settings().owner_called() {
            return sc_error!("only owner can unpause staking");
        }
        self.set_staking_paused(false);
        // TODO: event
        Ok(())
    }

    #[view(isStakeSalePaused)]
    #[storage_get("stake_sale_paused")]
    fn is_stake_sale_paused(&self) -> bool;

    #[storage_set("stake_sale_paused")]
    fn set_stake_sale_paused(&self, stake_sale_paused: bool);

    #[endpoint(pauseStakeSale)]
    fn pause_stake_sale(&self) -> SCResult<()>{
        if !self.settings().owner_called() {
            return sc_error!("only owner can pause stake sale");
        }
        self.set_stake_sale_paused(true);
        // TODO: event
        Ok(())
    }

    #[endpoint(unpauseStakeSale)]
    fn unpause_stake_sale(&self) -> SCResult<()>{
        if !self.settings().owner_called() {
            return sc_error!("only owner can unpause stake sale");
        }
        self.set_stake_sale_paused(false);
        // TODO: event
        Ok(())
    }

}
