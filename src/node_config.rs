
use crate::bls_key::*;
use crate::node_state::*;

use crate::settings::*;
use crate::node_activation::*;
use crate::user_stake::*;

imports!();

/// Indicates how we express the percentage of rewards that go to the node.
/// Since we cannot have floating point numbers, we use fixed point with this denominator.
/// Percents + 2 decimals -> 10000.
pub static SERVICE_FEE_DENOMINATOR: usize = 10000;

/// This module manages the validator node info:
/// - how many nodes there are,
/// - how much they need to stake and 
/// - what BLS keys they have.
/// 
#[elrond_wasm_derive::module(NodeConfigModuleImpl)]
pub trait NodeModule {

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    #[module(UserStakeModuleImpl)]
    fn user_stake(&self) -> UserStakeModuleImpl<T, BigInt, BigUint>;

    #[module(NodeActivationModuleImpl)]
    fn node_activation(&self) -> NodeActivationModuleImpl<T, BigInt, BigUint>;

    /// The proportion of rewards that goes to the owner as compensation for running the nodes.
    /// 10000 = 100%.
    #[view]
    #[storage_get("service_fee")]
    fn getServiceFee(&self) -> BigUint;

    #[private]
    #[storage_set("service_fee")]
    fn _set_service_fee(&self, service_fee: usize);

    /// The stake per node can be changed by the owner.
    /// It does not get set in the contructor, so the owner has to manually set it after the contract is deployed.
    fn setServiceFee(&self, service_fee_per_10000: usize) -> Result<(), &str> {
        if self.get_caller() != self.settings().getContractOwner() {
            return Err("only owner can change service fee"); 
        }

        if service_fee_per_10000 > SERVICE_FEE_DENOMINATOR {
            return Err("node share out of range");
        }

        // check that all nodes idle
        if !self.allNodesIdle() {
            return Err("cannot change service fee while at least one node is active");
        }

        self._set_service_fee(service_fee_per_10000);
        Ok(())
    }
    
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
    fn setStakePerNode(&self, node_activation: &BigUint) -> Result<(), &str> {
        if self.get_caller() != self.settings().getContractOwner() {
            return Err("only owner can change stake per node"); 
        }

        // check that all nodes idle
        if !self.allNodesIdle() {
            return Err("cannot change stake per node while at least one node is active");
        }

        self._set_stake_per_node(&node_activation);
        Ok(())
    }
    
    /// The number of nodes that will run with the contract stake, as configured by the owner.
    #[view]
    #[storage_get("num_nodes")]
    fn getNumNodes(&self) -> usize;

    #[private]
    #[storage_set("num_nodes")]
    fn _set_num_nodes(&self, num_nodes: usize);

    /// Each node gets a node id. This is in order to be able to iterate over their data.
    /// This is a mapping from node BLS key to node id.
    /// The key is the bytes "node_id" concatenated with the BLS key. The value is the node id.
    /// Ids start from 1 because 0 means unset of None.
    #[view]
    #[storage_get("node_bls_to_id")]
    fn getNodeId(&self, bls_key: &BLSKey) -> usize;

    #[private]
    #[storage_set("node_bls_to_id")]
    fn _set_node_bls_to_id(&self, bls_key: &BLSKey, node_id: usize);

    #[view]
    #[storage_get("node_id_to_bls")]
    fn _get_node_id_to_bls(&self, node_id: usize) -> BLSKey;

    #[private]
    #[storage_set("node_id_to_bls")]
    fn _set_node_id_to_bls(&self, node_id: usize, bls_key: &BLSKey);

    /// Current state of node: inactive, active, deleted, etc.
    #[storage_get("n_state")]
    fn _get_node_state(&self, user_id: usize) -> NodeState;

    #[private]
    #[storage_set("n_state")]
    fn _set_node_state(&self, user_id: usize, node_state: NodeState);

    #[view]
    fn allNodesIdle(&self) -> bool {
        let mut i = self.getNumNodes();
        while i > 0 {
            let node_state = self._get_node_state(i);
            if node_state != NodeState::Inactive && node_state != NodeState::Removed {
                return false;
            }
            i -= 1;
        }

        true
    }

    #[view]
    fn getAllNodeStates(&self) -> Vec<Vec<u8>> {
        let num_nodes = self.getNumNodes();
        let mut result: Vec<Vec<u8>> = Vec::new();
        for i in 1..num_nodes+1 {
            let bls = self._get_node_id_to_bls(i);
            result.push(bls.to_vec());
            let state = self._get_node_state(i);
            result.push([state.to_u8()].to_vec());
        }
        result
    }

    /// The number of nodes that will run with the contract stake is configured by the owner.
    /// It does not get set in the contructor, so the owner has to manually set it after the contract is deployed.
    /// Important: it has to be called BEFORE setting the BLS keys.
    fn addNodes(&self, #[var_args] bls_keys: Vec<BLSKey>) -> Result<(), &str> {
        if self.get_caller() != self.settings().getContractOwner() {
            return Err("only owner can add nodes"); 
        }

        let mut num_nodes = self.getNumNodes();

        for bls_key in bls_keys.iter() {
            let existing_node_id = self.getNodeId(bls_key);
            if existing_node_id == 0 {
                num_nodes += 1;
                let new_node_id = num_nodes;
                self._set_node_bls_to_id(bls_key, new_node_id);
                self._set_node_id_to_bls(new_node_id, bls_key);
                self._set_node_state(new_node_id, NodeState::Inactive);
            } else if self._get_node_state(existing_node_id) == NodeState::Removed {
                self._set_node_state(existing_node_id, NodeState::Inactive);
            } else {
                return Err("node already registered"); 
            }
        }
       
        self._set_num_nodes(num_nodes);
        Ok(())
    }

    fn removeNodes(&self, #[var_args] bls_keys: Vec<BLSKey>) -> Result<(), &str> {
        if self.get_caller() != self.settings().getContractOwner() {
            return Err("only owner can remove nodes"); 
        }

        for bls_key in bls_keys.iter() {
            let node_id = self.getNodeId(bls_key);
            if node_id == 0 {
                return Err("node not registered");
            }
            if self._get_node_state(node_id) != NodeState::Inactive {
                return Err("only inactive nodes can be removed");
            }
            self._set_node_state(node_id, NodeState::Removed);
        }

        Ok(())
    }

}
