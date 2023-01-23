use crate::auction_proxy;
use node_storage::{
    node_config::NodeIndexArrayVec,
    types::{BLSKey, BLSSignature, BLSStatusMultiArg, NodeState},
};

elrond_wasm::imports!();

#[elrond_wasm::derive::module]
pub trait NodeActivationModule:
    node_storage::node_config::NodeConfigModule
    + user_fund_storage::user_data::UserDataModule
    + user_fund_storage::fund_module::FundModule
    + user_fund_storage::fund_view_module::FundViewModule
    + user_fund_storage::fund_transf_module::FundTransformationsModule
    + crate::settings::SettingsModule
    + crate::reset_checkpoint_state::ResetCheckpointStateModule
    + crate::rewards_state::RewardStateModule
    + crate::user_stake_state::UserStakeStateModule
    + crate::events::EventsModule
{
    #[proxy]
    fn auction_proxy(&self, to: ManagedAddress) -> auction_proxy::Proxy<Self::Api>;

    /// Owner activates specific nodes.
    #[only_owner]
    #[endpoint(stakeNodes)]
    fn stake_nodes(
        &self,
        amount_to_stake: BigUint,
        bls_keys: MultiValueManagedVec<Self::Api, BLSKey<Self::Api>>,
    ) {
        require!(
            !self.is_bootstrap_mode(),
            "cannot stake nodes in bootstrap mode"
        );

        require!(
            !self.is_global_op_in_progress(),
            "node operations are temporarily paused as checkpoint is reset"
        );

        require!(
            self.total_unprotected() >= amount_to_stake,
            "not enough funds in contract to stake nodes"
        );

        self.validate_owner_stake_share();

        let mut node_ids = NodeIndexArrayVec::new();
        let mut bls_keys_signatures: MultiValueEncoded<
            Self::Api,
            MultiValue2<BLSKey<Self::Api>, BLSSignature<Self::Api>>,
        > = MultiValueEncoded::new();

        for bls_key in bls_keys.iter() {
            let node_id = self.get_node_id(&bls_key);
            require!(node_id != 0, "unknown node provided");

            require!(
                self.get_node_state(node_id) == NodeState::Inactive,
                "node must be inactive"
            );

            node_ids.push(node_id);
            let bls_signature = self.get_node_signature(node_id);
            bls_keys_signatures.push((bls_key, bls_signature).into());

            self.set_node_state(node_id, NodeState::PendingActivation);
        }

        self.perform_stake_nodes(node_ids, bls_keys_signatures, amount_to_stake);
    }

    fn perform_stake_nodes(
        &self,
        node_ids: NodeIndexArrayVec,
        bls_keys_signatures: MultiValueEncoded<
            MultiValue2<BLSKey<Self::Api>, BLSSignature<Self::Api>>,
        >,
        amount_to_stake: BigUint,
    ) {
        let num_nodes = node_ids.len();
        // send all stake to auction contract
        let auction_contract_addr = self.get_auction_contract_address();

        self.auction_proxy(auction_contract_addr)
            .stake(num_nodes, bls_keys_signatures)
            .with_egld_transfer(amount_to_stake)
            .async_call()
            .with_callback(self.callbacks().auction_stake_callback(node_ids))
            .call_and_exit()
    }

    /// Only finalize activation if we got confirmation from the auction contract.
    /// `#[callback]` also has be declared in lib.rs for the moment.
    #[callback]
    fn auction_stake_callback(
        &self,
        node_ids: NodeIndexArrayVec,
        #[call_result] call_result: ManagedAsyncCallResult<
            MultiValueEncoded<BLSStatusMultiArg<Self::Api>>,
        >,
    ) {
        match call_result {
            ManagedAsyncCallResult::Ok(node_status_args) => {
                let (node_ids_ok, node_ids_fail) =
                    self.split_node_ids_by_err(node_ids, node_status_args);
                self.auction_stake_callback_ok(&node_ids_ok);
                self.auction_stake_callback_fail(
                    &node_ids_fail,
                    &ManagedBuffer::from(b"staking failed for some nodes"),
                );
            }
            ManagedAsyncCallResult::Err(error) => {
                self.auction_stake_callback_fail(&node_ids, &error.err_msg)
            }
        }
    }

    fn auction_stake_callback_ok(&self, node_ids: &NodeIndexArrayVec) {
        if node_ids.is_empty() {
            return;
        }

        // set nodes to Active
        for &node_id in node_ids.iter() {
            self.set_node_state(node_id, NodeState::Active);
        }

        // log event (no data)
        // TODO: log BLS keys of nodes in data
        self.stake_node_ok_event();
    }

    fn auction_stake_callback_fail(&self, node_ids: &NodeIndexArrayVec, err_msg: &ManagedBuffer) {
        if node_ids.is_empty() {
            return;
        }

        // set nodes to Inactive
        for &node_id in node_ids.iter() {
            self.set_node_state(node_id, NodeState::Inactive);
        }

        // log failure event (no data)
        self.stake_node_fail_event(err_msg);
    }

    // UNSTAKE

    /// Unstakes from the auction smart contract.
    /// The nodes will stop receiving rewards, but stake cannot be yet reclaimed.
    /// This operation is performed by the owner.
    /// Does not unstake tokens.
    #[only_owner]
    #[endpoint(unStakeNodes)]
    fn unstake_nodes_endpoint(&self, bls_keys: MultiValueManagedVec<Self::Api, BLSKey<Self::Api>>) {
        self.unstake_nodes(false, bls_keys)
    }

    /// Unstakes from the auction smart contract.
    /// The nodes will stop receiving rewards, but stake cannot be yet reclaimed.
    /// This operation is performed by the owner.
    /// Also unstakes tokens.
    #[only_owner]
    #[endpoint(unStakeNodesAndTokens)]
    fn unstake_nodes_and_tokens_endpoint(
        &self,
        bls_keys: MultiValueManagedVec<Self::Api, BLSKey<Self::Api>>,
    ) {
        self.unstake_nodes(true, bls_keys)
    }

    fn unstake_nodes(
        &self,
        unstake_tokens: bool,
        bls_keys: MultiValueManagedVec<Self::Api, BLSKey<Self::Api>>,
    ) {
        require!(
            !self.is_global_op_in_progress(),
            "node operations are temporarily paused as checkpoint is reset"
        );

        let mut node_ids = NodeIndexArrayVec::new();
        for bls_key in bls_keys.iter() {
            let node_id = self.get_node_id(&bls_key);
            require!(node_id != 0, "unknown node provided");
            node_ids.push(node_id);
        }

        self.perform_unstake_nodes(unstake_tokens, node_ids, bls_keys)
    }

    fn perform_unstake_nodes(
        &self,
        unstake_tokens: bool,
        node_ids: NodeIndexArrayVec,
        bls_keys: MultiValueManagedVec<BLSKey<Self::Api>>,
    ) {
        // convert node state to PendingDeactivation
        for &node_id in node_ids.iter() {
            require!(
                self.get_node_state(node_id) == NodeState::Active,
                "node not active"
            );

            self.set_node_state(node_id, NodeState::PendingDeactivation);
        }

        // send unstake command to Auction SC
        let auction_contract_addr = self.get_auction_contract_address();
        if unstake_tokens {
            self.auction_proxy(auction_contract_addr)
                .unstake(bls_keys.clone())
                .async_call()
                .with_callback(self.callbacks().auction_unstake_callback(node_ids))
                .call_and_exit()
        } else {
            self.auction_proxy(auction_contract_addr)
                .unstake_nodes(bls_keys)
                .async_call()
                .with_callback(self.callbacks().auction_unstake_callback(node_ids))
                .call_and_exit()
        }
    }

    /// Only finalize deactivation if we got confirmation from the auction contract.
    /// `#[callback]` also has be declared in lib.rs for the moment.
    #[callback]
    fn auction_unstake_callback(
        &self,
        node_ids: NodeIndexArrayVec,
        #[call_result] call_result: ManagedAsyncCallResult<
            MultiValueEncoded<BLSStatusMultiArg<Self::Api>>,
        >,
    ) {
        match call_result {
            ManagedAsyncCallResult::Ok(node_status_args) => {
                let (node_ids_ok, node_ids_fail) =
                    self.split_node_ids_by_err(node_ids, node_status_args);
                self.auction_unstake_callback_ok(&node_ids_ok);
                self.auction_unstake_callback_fail(
                    &node_ids_fail,
                    &ManagedBuffer::from(b"unstaking failed for some nodes"),
                );
            }
            ManagedAsyncCallResult::Err(error) => {
                self.auction_unstake_callback_fail(&node_ids, &error.err_msg)
            }
        }
    }

    fn auction_unstake_callback_ok(&self, node_ids: &NodeIndexArrayVec) {
        if node_ids.is_empty() {
            return;
        }

        // set nodes to UnBondPeriod + save current block nonce
        let bl_nonce = self.blockchain().get_block_nonce();
        for &node_id in node_ids.iter() {
            self.set_node_state(node_id, NodeState::UnBondPeriod { started: bl_nonce });
        }

        // log event (no data)
        // TODO: log BLS keys of nodes in data
        self.unstake_node_ok_event();
    }

    /// Owner can retry a callback in case of callback failure.
    /// Warning: misuse can lead to state inconsistency.
    #[only_owner]
    #[endpoint(forceNodeUnBondPeriod)]
    fn force_node_unbond_period(
        &self,
        bls_keys: MultiValueManagedVec<Self::Api, BLSKey<Self::Api>>,
    ) {
        for bls_key in bls_keys.iter() {
            let node_id = self.get_node_id(&bls_key);
            require!(node_id != 0, "unknown node provided");
            self.set_node_state(node_id, NodeState::UnBondPeriod { started: 0 });
        }
    }

    fn auction_unstake_callback_fail(&self, node_ids: &NodeIndexArrayVec, err_msg: &ManagedBuffer) {
        if node_ids.is_empty() {
            return;
        }

        // revert nodes to Active
        for &node_id in node_ids.iter() {
            self.set_node_state(node_id, NodeState::Active);
        }

        // log failure event (no data)
        self.unstake_node_fail_event(err_msg);
    }

    // UNBOND
    /// Calls unbond for all provided nodes. Will fail if node cannot be unbonded.
    #[only_owner]
    #[endpoint(unBondNodes)]
    fn unbond_specific_nodes_endpoint(
        &self,
        bls_keys: MultiValueManagedVec<Self::Api, BLSKey<Self::Api>>,
    ) {
        require!(
            !self.is_global_op_in_progress(),
            "node operations are temporarily paused as checkpoint is reset"
        );

        require!(!bls_keys.is_empty(), "no BLS keys provided");

        let mut node_ids = NodeIndexArrayVec::new();
        for bls_key in bls_keys.iter() {
            let node_id = self.get_node_id(&bls_key);
            require!(node_id != 0, "unknown node provided");
            require!(
                self.prepare_node_for_unbond_if_possible(node_id),
                "node cannot be unbonded"
            );
            node_ids.push(node_id);
        }

        self.perform_unbond(node_ids, bls_keys);
    }

    /// Calls unbond for all nodes that are in the unbond period and are due.
    /// Nothing happens if no nodes can be unbonded.
    #[only_owner]
    #[endpoint(unBondAllPossibleNodes)]
    fn unbond_all_possible_nodes(&self) {
        require!(
            !self.is_global_op_in_progress(),
            "node operations are temporarily paused as checkpoint is reset"
        );

        let mut node_id = self.num_nodes().get();
        let mut node_ids = NodeIndexArrayVec::new();
        let mut bls_keys = MultiValueManagedVec::<Self::Api, BLSKey<Self::Api>>::new();
        while node_id >= 1 {
            if self.prepare_node_for_unbond_if_possible(node_id) {
                node_ids.push(node_id);
                bls_keys.push(self.get_node_id_to_bls(node_id));
            }

            node_id -= 1;
        }

        if !node_ids.is_empty() {
            self.perform_unbond(node_ids, bls_keys);
        }
    }

    fn prepare_node_for_unbond_if_possible(&self, node_id: usize) -> bool {
        if let NodeState::UnBondPeriod { started } = self.get_node_state(node_id) {
            self.set_node_state(
                node_id,
                NodeState::PendingUnBond {
                    unbond_started: started,
                },
            );
            return true;
        }

        false
    }

    fn perform_unbond(
        &self,
        node_ids: NodeIndexArrayVec,
        bls_keys: MultiValueManagedVec<BLSKey<Self::Api>>,
    ) {
        // send unbond command to Auction SC
        let auction_contract_addr = self.get_auction_contract_address();
        self.auction_proxy(auction_contract_addr)
            .unbond_nodes(bls_keys)
            .async_call()
            .with_callback(self.callbacks().auction_unbond_callback(node_ids))
            .call_and_exit()
    }

    /// Only finalize deactivation if we got confirmation from the auction contract.
    /// `#[callback]` also has be declared in lib.rs for the moment.
    #[callback]
    fn auction_unbond_callback(
        &self,
        node_ids: NodeIndexArrayVec,
        #[call_result] call_result: ManagedAsyncCallResult<
            MultiValueEncoded<BLSStatusMultiArg<Self::Api>>,
        >,
    ) {
        match call_result {
            ManagedAsyncCallResult::Ok(node_status_args) => {
                let (node_ids_ok, node_ids_fail) =
                    self.split_node_ids_by_err(node_ids, node_status_args);
                self.auction_unbond_callback_ok(&node_ids_ok);
                self.auction_unbond_callback_fail(
                    &node_ids_fail,
                    &ManagedBuffer::from(b"unbonding failed for some nodes"),
                );
            }
            ManagedAsyncCallResult::Err(error) => {
                self.auction_unbond_callback_fail(&node_ids, &error.err_msg)
            }
        }
    }

    fn auction_unbond_callback_ok(&self, node_ids: &NodeIndexArrayVec) {
        if node_ids.is_empty() {
            return;
        }

        // set nodes to Inactive + reset unstake nonce since it is no longer needed
        for &node_id in node_ids.iter() {
            self.set_node_state(node_id, NodeState::Inactive);
        }

        // log event (no data)
        // TODO: log BLS keys of nodes in data
        self.unbond_node_ok_event();
    }

    fn auction_unbond_callback_fail(&self, node_ids: &NodeIndexArrayVec, err_msg: &ManagedBuffer) {
        if node_ids.is_empty() {
            return;
        }

        // revert nodes to UnBondPeriod
        for &node_id in node_ids.iter() {
            if let NodeState::PendingUnBond { unbond_started } = self.get_node_state(node_id) {
                self.set_node_state(
                    node_id,
                    NodeState::UnBondPeriod {
                        started: unbond_started,
                    },
                );
            } else {
                sc_panic!("node not pending unbond");
            }
        }

        // log failure event (no data)
        self.unbond_node_fail_event(err_msg);
    }

    /// Claims from auction SC funds that were sent but are not required to run the nodes.
    #[only_owner]
    #[endpoint(claimUnusedFunds)]
    fn claim_unused_funds(&self) {
        require!(
            !self.is_global_op_in_progress(),
            "node operations are temporarily paused as checkpoint is reset"
        );

        // send claim command to Auction SC
        let auction_contract_addr = self.get_auction_contract_address();
        self.auction_proxy(auction_contract_addr)
            .claim()
            .async_call()
            .call_and_exit()
    }

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(unJailNodes)]
    fn unjail_nodes(
        &self,
        bls_keys: MultiValueManagedVec<Self::Api, BLSKey<Self::Api>>,
        #[payment] fine_payment: BigUint,
    ) {
        // validation only
        for bls_key in bls_keys.iter() {
            let node_id = self.get_node_id(&bls_key);
            require!(node_id != 0, "unknown node provided");
            require!(
                self.get_node_state(node_id) == NodeState::Active,
                "node must be active"
            );
        }

        // send unJail command to Auction SC
        let auction_contract_addr = self.get_auction_contract_address();
        self.auction_proxy(auction_contract_addr)
            .unjail(bls_keys)
            .with_egld_transfer(fine_payment)
            .async_call()
            .call_and_exit()
    }

    #[endpoint(unStakeTokens)]
    fn unstake_tokens(&self, amount: BigUint) {
        self.unstake_tokens_event(&amount);
        let auction_contract_addr = self.get_auction_contract_address();
        self.auction_proxy(auction_contract_addr)
            .unstake_tokens(&amount)
            .async_call()
            .call_and_exit()
    }

    #[endpoint(unBondTokens)]
    fn unbond_tokens(&self, amount: BigUint) {
        self.unbond_tokens_event(&amount);
        let auction_contract_addr = self.get_auction_contract_address();
        self.auction_proxy(auction_contract_addr)
            .unbond_tokens(&amount)
            .async_call()
            .call_and_exit()
    }
}
