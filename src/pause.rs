
use crate::settings::*;

imports!();

/// The module deals with temporarily pausing certain operations.
/// 
#[elrond_wasm_derive::module(PauseModuleImpl)]
pub trait PauseModule {

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    #[view]
    #[storage_get("staking_paused")]
    fn isStakingPaused(&self) -> bool;

    #[private]
    #[storage_set("staking_paused")]
    fn _set_staking_paused(&self, staking_paused: bool);

    fn pauseStaking(&self) -> Result<(), SCError>{
        if !self.settings()._owner_called() {
            return sc_error!("only owner can pause staking");
        }
        self._set_staking_paused(true);
        // TODO: event
        Ok(())
    }

    fn unpauseStaking(&self) -> Result<(), SCError>{
        if !self.settings()._owner_called() {
            return sc_error!("only owner can unpause staking");
        }
        self._set_staking_paused(false);
        // TODO: event
        Ok(())
    }

    #[view]
    #[storage_get("stake_sale_paused")]
    fn isStakeSalePaused(&self) -> bool;

    #[private]
    #[storage_set("stake_sale_paused")]
    fn _set_stake_sale_paused(&self, stake_sale_paused: bool);

    fn pauseStakeSale(&self) -> Result<(), SCError>{
        if !self.settings()._owner_called() {
            return sc_error!("only owner can pause stake sale");
        }
        self._set_stake_sale_paused(true);
        // TODO: event
        Ok(())
    }

    fn unpauseStakeSale(&self) -> Result<(), SCError>{
        if !self.settings()._owner_called() {
            return sc_error!("only owner can unpause stake sale");
        }
        self._set_stake_sale_paused(false);
        // TODO: event
        Ok(())
    }

}
