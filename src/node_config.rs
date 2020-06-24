
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
    fn setServiceFee(&self, service_fee_per_10000: usize) -> Result<(), SCError> {
        if !self.settings()._owner_called() {
            return sc_error!("only owner can change service fee"); 
        }

        if service_fee_per_10000 > SERVICE_FEE_DENOMINATOR {
            return sc_error!("node share out of range");
        }

        // check that all nodes idle
        if !self.allNodesIdle() {
            return sc_error!("cannot change service fee while at least one node is active");
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
    fn setStakePerNode(&self, node_activation: &BigUint) -> Result<(), SCError> {
        if !self.settings()._owner_called() {
            return sc_error!("only owner can change stake per node"); 
        }

        // check that all nodes idle
        if !self.allNodesIdle() {
            return sc_error!("cannot change stake per node while at least one node is active");
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

    #[private]
    #[storage_get("node_id_to_bls")]
    fn _get_node_id_to_bls(&self, node_id: usize) -> BLSKey;

    #[private]
    #[storage_set("node_id_to_bls")]
    fn _set_node_id_to_bls(&self, node_id: usize, bls_key: &BLSKey);

    #[private]
    #[storage_get("node_signature")]
    fn _get_node_signature(&self, node_id: usize) -> BLSSignature;

    #[private]
    #[storage_set("node_signature")]
    fn _set_node_signature(&self, node_id: usize, node_signature: BLSSignature);

    #[view]
    fn getNodeSignature(&self, bls_key: BLSKey) -> OptionalResult<BLSSignature> {
        let node_id = self.getNodeId(&bls_key);
        if node_id == 0 {
            OptionalResult::None
        } else {
            OptionalResult::Some(self._get_node_signature(node_id))
        }
    }

    /// Current state of node: inactive, active, deleted, etc.
    #[private]
    #[storage_get("node_state")]
    fn _get_node_state(&self, node_id: usize) -> NodeState;

    #[private]
    #[storage_set("node_state")]
    fn _set_node_state(&self, node_id: usize, node_state: NodeState);

    #[view]
    fn getNodeState(&self, bls_key: BLSKey) -> NodeState {
        let node_id = self.getNodeId(&bls_key);
        if node_id == 0 {
            NodeState::Removed
        } else {
            self._get_node_state(node_id)
        }
    }

    /// True if all nodes are either inactive or removed.
    /// Some operations (like setServiceFee and setStakePerNode) can only be performed when all nodes are idle.
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
    fn getAllNodeStates(&self) -> MultiResultVec<Vec<u8>> {
        let num_nodes = self.getNumNodes();
        let mut result: Vec<Vec<u8>> = Vec::new();
        for i in 1..num_nodes+1 {
            let bls = self._get_node_id_to_bls(i);
            result.push(bls.to_vec());
            let state = self._get_node_state(i);
            result.push([state.to_u8()].to_vec());
        }
        result.into()
    }

    /// Block timestamp when unbond happened. 0 if not in unbond period.
    #[private]
    #[storage_get("node_bl_nonce_of_unstake")]
    fn _get_node_bl_nonce_of_unstake(&self, node_id: usize) -> u64;

    #[private]
    #[storage_set("node_bl_nonce_of_unstake")]
    fn _set_node_bl_nonce_of_unstake(&self, node_id: usize, bl_nonce_of_unstake: u64);

    fn getNodeBlockNonceOfUnstake(&self, bls_key: BLSKey) -> u64 {
        let node_id = self.getNodeId(&bls_key);
        if node_id == 0 {
            0
        } else {
            self._get_node_bl_nonce_of_unstake(node_id)
        }
    }

    /// The number of nodes that will run with the contract stake is configured by the owner.
    /// It does not get set in the contructor, so the owner has to manually set it after the contract is deployed.
    /// Important: it has to be called BEFORE setting the BLS keys.
    fn addNodes(&self, 
            #[var_args] bls_keys_signatures: Vec<Vec<u8>>)
        -> Result<(), SCError> {

        if !self.settings()._owner_called() {
            return sc_error!("only owner can add nodes"); 
        }

        let mut num_nodes = self.getNumNodes();
        if bls_keys_signatures.len() % 2 != 0 {
            return sc_error!("even number of arguments expected"); 
        }

        // TODO: handle arguments more elegantly,
        // once elrond-wasm supports more complex multi-arg definitions
        let mut node_id = 0usize;
        for (i, arg) in bls_keys_signatures.iter().enumerate() {
            if i % 2 == 0 {
                let bls_key = BLSKey::from_bytes(arg)?;
                node_id = self.getNodeId(&bls_key);
                if node_id == 0 {
                    num_nodes += 1;
                    node_id = num_nodes;
                    self._set_node_bls_to_id(&bls_key, node_id);
                    self._set_node_id_to_bls(node_id, &bls_key);
                    self._set_node_state(node_id, NodeState::Inactive);
                } else if self._get_node_state(node_id) == NodeState::Removed {
                    self._set_node_state(node_id, NodeState::Inactive);
                } else {
                    return sc_error!("node already registered"); 
                }
            } else {
                // check signature lengths
                if arg.len() != BLS_SIGNATURE_BYTE_LENGTH {
                    return sc_error!("wrong size BLS signature");
                }
                let signature = BLSSignature::from_slice(arg.as_slice());
                self._set_node_signature(node_id, signature)
            }
        }
       
        self._set_num_nodes(num_nodes);
        Ok(())
    }

    fn removeNodes(&self, #[var_args] bls_keys: Vec<BLSKey>) -> Result<(), SCError> {
        if !self.settings()._owner_called() {
            return sc_error!("only owner can remove nodes"); 
        }

        for bls_key in bls_keys.iter() {
            let node_id = self.getNodeId(bls_key);
            if node_id == 0 {
                return sc_error!("node not registered");
            }
            if self._get_node_state(node_id) != NodeState::Inactive {
                return sc_error!("only inactive nodes can be removed");
            }
            self._set_node_state(node_id, NodeState::Removed);
        }

        Ok(())
    }

    /// Called when a user decides to forcefully unstake own share.
    /// Finds enough nodes to cover requested stake.
    /// Both node ids and node BLS keys are required, separately.
    #[private]
    fn _find_nodes_for_unstake(&self, requested_stake: &BigUint) -> (Vec<usize>, Vec<BLSKey>) {

        let mut node_ids: Vec<usize> = Vec::new();
        let mut bls_keys: Vec<BLSKey> = Vec::new();
        let mut i = self.getNumNodes();
        let mut node_stake = BigUint::zero();
        let stake_per_node = self.getStakePerNode();
        while i > 0 && &node_stake < requested_stake {
            if let NodeState::Active = self._get_node_state(i) {
                node_stake += &stake_per_node;
                node_ids.push(i);
                bls_keys.push(self._get_node_id_to_bls(i));
            }
            i -= 1;
        }

        (node_ids, bls_keys)
    }

    #[private]
    fn _split_node_ids_by_err(&self, 
            mut node_ids: Vec<usize>, 
            node_fail_map_raw: VarArgs<Vec<u8>>)
        -> Result<(Vec<usize>, Vec<usize>), SCError> {

        if node_fail_map_raw.len() == 0 {
            return Ok((node_ids, Vec::with_capacity(0)));
        }

        if node_fail_map_raw.len() % 2 != 0 {
            return sc_error!("even number of arguments expected in auction callback");
        }

        let mut failed_node_ids: Vec<usize> = Vec::new();

        let mut node_id = 0usize;
        for (i, arg) in node_fail_map_raw.iter().enumerate() {
            if i % 2 == 0 {
                let bls_key = BLSKey::from_bytes(arg)?;
                node_id = self.getNodeId(&bls_key); 
            } else {
                if arg.len() != 1 {
                    return sc_error!("node status expected as one byte");
                }
                if arg[0] > 0 {
                    // error
                    if let Some(pos) = node_ids.iter().position(|x| *x == node_id) {
                        node_ids.swap_remove(pos);
                        failed_node_ids.push(node_id);
                    }
                }
            }
        }

        Ok((node_ids, failed_node_ids))
    }
}
