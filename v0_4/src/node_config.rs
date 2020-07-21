
use crate::bls_key::*;
use crate::node_state::*;

use crate::settings::*;
use crate::node_activation::*;
use crate::user_stake::*;

imports!();

/// Indicates how we express the percentage of rewards that go to the node.
/// Since we cannot have floating point numbers, we use fixed point with this denominator.
/// Percents + 2 decimals -> 10000.
pub static PERCENTAGE_DENOMINATOR: usize = 10000;

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
    #[view(getServiceFee)]
    #[storage_get("service_fee")]
    fn get_service_fee(&self) -> BigUint;

    #[storage_set("service_fee")]
    fn set_service_fee(&self, service_fee: usize);

    /// The stake per node can be changed by the owner.
    /// It does not get set in the contructor, so the owner has to manually set it after the contract is deployed.
    #[endpoint(setServiceFee)]
    fn set_service_fee_endpoint(&self, service_fee_per_10000: usize) -> SCResult<()> {
        if !self.settings().owner_called() {
            return sc_error!("only owner can change service fee"); 
        }

        if service_fee_per_10000 > PERCENTAGE_DENOMINATOR {
            return sc_error!("service fee out of range");
        }

        // check that all nodes idle
        if !self.all_nodes_idle() {
            return sc_error!("cannot change service fee while at least one node is active");
        }

        self.set_service_fee(service_fee_per_10000);
        Ok(())
    }
    
    /// How much stake has to be provided per validator node.
    /// After genesis this sum is fixed to 2,500,000 ERD, but at some point bidding will happen.
    #[view(getStakePerNode)]
    #[storage_get("stake_per_node")]
    fn get_stake_per_node(&self) -> BigUint;

    #[storage_set("stake_per_node")]
    fn set_stake_per_node(&self, spn: &BigUint);

    /// The stake per node can be changed by the owner.
    /// It does not get set in the contructor, so the owner has to manually set it after the contract is deployed.
    #[endpoint(setStakePerNode)]
    fn set_stake_per_node_endpoint(&self, node_activation: &BigUint) -> SCResult<()> {
        if !self.settings().owner_called() {
            return sc_error!("only owner can change stake per node"); 
        }

        // check that all nodes idle
        if !self.all_nodes_idle() {
            return sc_error!("cannot change stake per node while at least one node is active");
        }

        self.set_stake_per_node(&node_activation);
        Ok(())
    }
    
    /// The number of nodes that will run with the contract stake, as configured by the owner.
    #[view(getNumNodes)]
    #[storage_get("num_nodes")]
    fn get_num_nodes(&self) -> usize;

    #[storage_get_mut("num_nodes")]
    fn get_mut_num_nodes(&self) -> mut_storage!(usize);

    /// Each node gets a node id. This is in order to be able to iterate over their data.
    /// This is a mapping from node BLS key to node id.
    /// The key is the bytes "node_id" concatenated with the BLS key. The value is the node id.
    /// Ids start from 1 because 0 means unset of None.
    #[view(getNodeId)]
    #[storage_get("node_bls_to_id")]
    fn get_node_id(&self, bls_key: &BLSKey) -> usize;

    #[storage_set("node_bls_to_id")]
    fn set_node_bls_to_id(&self, bls_key: &BLSKey, node_id: usize);

    #[storage_get("node_id_to_bls")]
    fn get_node_id_to_bls(&self, node_id: usize) -> BLSKey;

    #[storage_set("node_id_to_bls")]
    fn set_node_id_to_bls(&self, node_id: usize, bls_key: &BLSKey);

    #[storage_get("node_signature")]
    fn get_node_signature(&self, node_id: usize) -> BLSSignature;

    #[storage_set("node_signature")]
    fn set_node_signature(&self, node_id: usize, node_signature: BLSSignature);

    #[view(getNodeSignature)]
    fn get_node_signature_endpoint(&self, bls_key: BLSKey) -> OptionalResult<BLSSignature> {
        let node_id = self.get_node_id(&bls_key);
        if node_id == 0 {
            OptionalResult::None
        } else {
            OptionalResult::Some(self.get_node_signature(node_id))
        }
    }

    /// Current state of node: inactive, active, deleted, etc.
    #[storage_get("node_state")]
    fn get_node_state(&self, node_id: usize) -> NodeState;

    #[storage_set("node_state")]
    fn set_node_state(&self, node_id: usize, node_state: NodeState);

    #[view(getNodeState)]
    fn get_node_state_endpoint(&self, bls_key: BLSKey) -> NodeState {
        let node_id = self.get_node_id(&bls_key);
        if node_id == 0 {
            NodeState::Removed
        } else {
            self.get_node_state(node_id)
        }
    }

    /// True if all nodes are either inactive or removed.
    /// Some operations (like setServiceFee and setStakePerNode) can only be performed when all nodes are idle.
    #[view(allNodesIdle)]
    fn all_nodes_idle(&self) -> bool {
        let mut i = self.get_num_nodes();
        while i > 0 {
            let node_state = self.get_node_state(i);
            if node_state != NodeState::Inactive && node_state != NodeState::Removed {
                return false;
            }
            i -= 1;
        }

        true
    }

    #[view(getAllNodeStates)]
    fn get_all_node_states(&self) -> MultiResultVec<Vec<u8>> {
        let num_nodes = self.get_num_nodes();
        let mut result: Vec<Vec<u8>> = Vec::new();
        for i in 1..num_nodes+1 {
            let bls = self.get_node_id_to_bls(i);
            result.push(bls.to_vec());
            let state = self.get_node_state(i);
            result.push([state.to_u8()].to_vec());
        }
        result.into()
    }

    #[view(getNodeBlockNonceOfUnstake)]
    fn get_node_bl_nonce_of_unstake_endpoint(&self, bls_key: BLSKey) -> OptionalResult<u64> {
        let node_id = self.get_node_id(&bls_key);
        if node_id == 0 {
            OptionalResult::None
        } else if let NodeState::UnBondPeriod{ started } = self.get_node_state(node_id) {
            OptionalResult::Some(started)
        } else {
            OptionalResult::None
        }
    }

    #[endpoint(addNodes)]
    fn add_nodes(&self, 
            #[var_args] bls_keys_signatures: VarArgs<MultiArg2<BLSKey, BLSSignature>>)
        -> SCResult<()> {

        if !self.settings().owner_called() {
            return sc_error!("only owner can add nodes"); 
        }

        let mut num_nodes = self.get_mut_num_nodes();
        for bls_sig_pair_arg in bls_keys_signatures.into_vec().into_iter() {
            let (bls_key, bls_sig) = bls_sig_pair_arg.into_tuple();
            let mut node_id = self.get_node_id(&bls_key);
            if node_id == 0 {
                *num_nodes += 1;
                node_id = *num_nodes;
                self.set_node_bls_to_id(&bls_key, node_id);
                self.set_node_id_to_bls(node_id, &bls_key);
                self.set_node_state(node_id, NodeState::Inactive);
                self.set_node_signature(node_id, bls_sig);
            } else if self.get_node_state(node_id) == NodeState::Removed {
                self.set_node_state(node_id, NodeState::Inactive);
                self.set_node_signature(node_id, bls_sig);
            } else {
                return sc_error!("node already registered"); 
            }
        }
        Ok(())
    }

    #[endpoint(removeNodes)]
    fn remove_nodes(&self, #[var_args] bls_keys: VarArgs<BLSKey>) -> SCResult<()> {
        if !self.settings().owner_called() {
            return sc_error!("only owner can remove nodes"); 
        }

        for bls_key in bls_keys.iter() {
            let node_id = self.get_node_id(bls_key);
            if node_id == 0 {
                return sc_error!("node not registered");
            }
            if self.get_node_state(node_id) != NodeState::Inactive {
                return sc_error!("only inactive nodes can be removed");
            }
            self.set_node_state(node_id, NodeState::Removed);
        }

        Ok(())
    }

    /// Called when a user decides to forcefully unstake own share.
    /// Finds enough nodes to cover requested stake.
    /// Both node ids and node BLS keys are required, separately.
    fn find_nodes_for_unstake(&self, requested_stake: &BigUint) -> (Vec<usize>, Vec<BLSKey>) {

        let mut node_ids: Vec<usize> = Vec::new();
        let mut bls_keys: Vec<BLSKey> = Vec::new();
        let mut i = self.get_num_nodes();
        let mut node_stake = BigUint::zero();
        let stake_per_node = self.get_stake_per_node();
        while i > 0 && &node_stake < requested_stake {
            if let NodeState::Active = self.get_node_state(i) {
                node_stake += &stake_per_node;
                node_ids.push(i);
                bls_keys.push(self.get_node_id_to_bls(i));
            }
            i -= 1;
        }

        (node_ids, bls_keys)
    }

    fn split_node_ids_by_err(&self, 
            mut node_ids: Vec<usize>, 
            node_status_args: VarArgs<BLSStatusMultiArg>)
        -> (Vec<usize>, Vec<usize>) {

        let mut failed_node_ids: Vec<usize> = Vec::new();
        for arg in node_status_args.into_vec().into_iter() {
            let (bls_key, status) = arg.into_tuple();
            if status != 0 {
                let node_id = self.get_node_id(&bls_key);
                // move node from ok nodes to failed ones
                if let Some(pos) = node_ids.iter().position(|x| *x == node_id) {
                    node_ids.swap_remove(pos);
                    failed_node_ids.push(node_id);
                }
            }
        }

        (node_ids, failed_node_ids)
    }
}
