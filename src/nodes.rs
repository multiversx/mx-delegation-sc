
use crate::bls_key::*;
use crate::settings::*;
use crate::stake_per_contract::*;
use crate::stake_per_user::*;

/// This module manages the validator node info:
/// - how many nodes there are,
/// - how much they need to stake and 
/// - what BLS keys they have.
/// 
#[elrond_wasm_derive::module(NodeModuleImpl)]
pub trait NodeModule {

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    #[module(UserStakeModuleImpl)]
    fn user_stake(&self) -> UserStakeModuleImpl<T, BigInt, BigUint>;

    #[module(ContractStakeModuleImpl)]
    fn contract_stake(&self) -> ContractStakeModuleImpl<T, BigInt, BigUint>;



    /// How much stake has to be provided per validator node.
    /// After genesis this sum is fixed to 2,500,000 ERD, but at some point bidding will happen.
    #[view]
    #[storage_get("stake_per_node")]
    fn getStakePerNode(&self) -> BigUint;

    #[private]
    #[storage_set("stake_per_node")]
    fn _set_stake_per_node(&self, spn: &BigUint);

    /// The stake per node can be changed by the owner.
    /// It does not get set in the contructor, so the owner has to manually set it after the contract is deployed.
    fn setStakePerNode(&self, stake_per_node: &BigUint) -> Result<(), &str> {
        if self.get_caller() != self.settings().getContractOwner() {
            return Err("only owner can change stake per node"); 
        }
        if !self.contract_stake().stakeState().is_open() {
            return Err("cannot change stake per node while active"); 
        }
        self._set_stake_per_node(&stake_per_node);
        Ok(())
    }

    /// Indicates how much stake the whole contract needs to accumulate before it can run.
    #[view]
    fn getExpectedStake(&self) -> BigUint {
        self.getStakePerNode() * BigUint::from(self.getNumNodes())
    }

    /// The number of nodes that will run with the contract stake, as configured by the owner.
    #[view]
    #[storage_get("num_nodes")]
    fn getNumNodes(&self) -> usize;

    #[private]
    #[storage_set("num_nodes")]
    fn _set_num_nodes(&self, num_nodes: usize);

    /// The number of nodes that will run with the contract stake is configured by the owner.
    /// It does not get set in the contructor, so the owner has to manually set it after the contract is deployed.
    /// Important: it has to be called BEFORE setting the BLS keys.
    fn setNumNodes(&self, num_nodes: usize) -> Result<(), &str> {
        if self.get_caller() != self.settings().getContractOwner() {
            return Err("only owner can change the number of nodes"); 
        }
        if !self.contract_stake().stakeState().is_open() {
            return Err("cannot change the number of nodes while active"); 
        }
        self._set_num_nodes(num_nodes);
        self._set_bls_keys(Vec::with_capacity(0)); // reset BLS keys
        Ok(())
    }

    /// Node BLS keys, as configured by the owner.
    /// Will yield multiple results, one result for each BLS key.
    /// Note: in storage they get concatenated and stored under a single key.
    #[view]
    #[storage_get("bls_keys")]
    fn getBlsKeys(&self) -> Vec<BLSKey>;

    #[private]
    #[storage_set("bls_keys")]
    fn _set_bls_keys(&self, bls_keys: Vec<BLSKey>);

    /// Convenience function for checking the number of BLS keys.
    /// It can be equal to the number of nodes or 0 if not yet configured.
    #[view]
    fn getNumBlsKeys(&self) -> usize {
        self.getBlsKeys().len()
    }

    /// The owner must set all the BLS keys of the nodes to run, before starting the validation.
    /// The function receives one argument for each BLS key.
    /// The number of arguments/BLS keys must match the number of nodes.
    /// Important: it has to be called AFTER setting the BLS keys.
    fn setBlsKeys(&self,
            #[multi(self.getNumNodes())] bls_keys: Vec<BLSKey>) -> Result<(), &str> {

        if self.get_caller() != self.settings().getContractOwner() {
            return Err("only owner can set BLS keys"); 
        }
        if !self.contract_stake().stakeState().is_open() {
            return Err("cannot change BLS keys while active"); 
        }
        
        self._set_bls_keys(bls_keys);

        Ok(())
    }

}