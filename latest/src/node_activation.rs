use crate::auction_proxy;
use node_storage::types::{BLSKey, BLSSignature, BLSStatusMultiArg, NodeState};

elrond_wasm::imports!();

#[elrond_wasm_derive::module]
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
    fn auction_proxy(&self, to: Address) -> auction_proxy::Proxy<Self::SendApi>;

    /// Owner activates specific nodes.
    #[endpoint(stakeNodes)]
    fn stake_nodes(
        &self,
        amount_to_stake: Self::BigUint,
        #[var_args] bls_keys: VarArgs<BLSKey>,
    ) -> SCResult<AsyncCall<Self::SendApi>> {
        only_owner!(self, "only owner allowed to stake nodes");

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

        self.validate_owner_stake_share()?;

        let mut node_ids = Vec::<usize>::with_capacity(bls_keys.len());
        let mut bls_keys_signatures: Vec<MultiArg2<BLSKey, BLSSignature>> = Vec::new();

        for bls_key in bls_keys.into_vec().into_iter() {
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

        Ok(self.perform_stake_nodes(node_ids, bls_keys_signatures.into(), amount_to_stake))
    }

    fn perform_stake_nodes(
        &self,
        node_ids: Vec<usize>,
        bls_keys_signatures: VarArgs<MultiArg2<BLSKey, BLSSignature>>,
        amount_to_stake: Self::BigUint,
    ) -> AsyncCall<Self::SendApi> {
        let num_nodes = node_ids.len();
        // send all stake to auction contract
        let auction_contract_addr = self.get_auction_contract_address();

        self.auction_proxy(auction_contract_addr)
            .with_token_transfer(TokenIdentifier::egld(), amount_to_stake)
            .stake(num_nodes, bls_keys_signatures)
            .async_call()
            .with_callback(self.callbacks().auction_stake_callback(node_ids))
    }

    /// Only finalize activation if we got confirmation from the auction contract.
    /// `#[callback]` also has be declared in lib.rs for the moment.
    #[callback]
    fn auction_stake_callback(
        &self,
        node_ids: Vec<usize>, // #[callback_arg]
        #[call_result] call_result: AsyncCallResult<MultiResultVec<BLSStatusMultiArg>>,
    ) -> SCResult<()> {
        match call_result {
            AsyncCallResult::Ok(node_status_args) => {
                let (node_ids_ok, node_ids_fail) =
                    self.split_node_ids_by_err(node_ids, node_status_args);
                self.auction_stake_callback_ok(node_ids_ok)?;
                self.auction_stake_callback_fail(
                    node_ids_fail,
                    &b"staking failed for some nodes"[..],
                )?;
                Ok(())
            }
            AsyncCallResult::Err(error) => {
                self.auction_stake_callback_fail(node_ids, error.err_msg.as_slice())
            }
        }
    }

    fn auction_stake_callback_ok(&self, node_ids: Vec<usize>) -> SCResult<()> {
        if node_ids.is_empty() {
            return Ok(());
        }

        // set nodes to Active
        for &node_id in node_ids.iter() {
            self.set_node_state(node_id, NodeState::Active);
        }

        // log event (no data)
        // TODO: log BLS keys of nodes in data
        self.stake_node_ok_event(());

        Ok(())
    }

    fn auction_stake_callback_fail(&self, node_ids: Vec<usize>, err_msg: &[u8]) -> SCResult<()> {
        if node_ids.is_empty() {
            return Ok(());
        }

        // set nodes to Inactive
        for &node_id in node_ids.iter() {
            self.set_node_state(node_id, NodeState::Inactive);
        }

        // log failure event (no data)
        self.stake_node_fail_event(err_msg);

        Ok(())
    }

    // UNSTAKE

    /// Unstakes from the auction smart contract.
    /// The nodes will stop receiving rewards, but stake cannot be yet reclaimed.
    /// This operation is performed by the owner.
    /// Does not unstake tokens.
    #[endpoint(unStakeNodes)]
    fn unstake_nodes_endpoint(
        &self,
        #[var_args] bls_keys: VarArgs<BLSKey>,
    ) -> SCResult<AsyncCall<Self::SendApi>> {
        self.unstake_nodes(false, bls_keys)
    }

    /// Unstakes from the auction smart contract.
    /// The nodes will stop receiving rewards, but stake cannot be yet reclaimed.
    /// This operation is performed by the owner.
    /// Also unstakes tokens.
    #[endpoint(unStakeNodesAndTokens)]
    fn unstake_nodes_and_tokens_endpoint(
        &self,
        #[var_args] bls_keys: VarArgs<BLSKey>,
    ) -> SCResult<AsyncCall<Self::SendApi>> {
        self.unstake_nodes(true, bls_keys)
    }

    fn unstake_nodes(
        &self,
        unstake_tokens: bool,
        bls_keys: VarArgs<BLSKey>,
    ) -> SCResult<AsyncCall<Self::SendApi>> {
        only_owner!(self, "only owner allowed to unstake nodes");

        require!(
            !self.is_global_op_in_progress(),
            "node operations are temporarily paused as checkpoint is reset"
        );

        let mut node_ids = Vec::<usize>::with_capacity(bls_keys.len());
        for bls_key in bls_keys.iter() {
            let node_id = self.get_node_id(bls_key);
            require!(node_id != 0, "unknown node provided");
            node_ids.push(node_id);
        }

        self.perform_unstake_nodes(unstake_tokens, node_ids, bls_keys.into_vec())
    }

    fn perform_unstake_nodes(
        &self,
        unstake_tokens: bool,
        node_ids: Vec<usize>,
        bls_keys: Vec<BLSKey>,
    ) -> SCResult<AsyncCall<Self::SendApi>> {
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
        let auction_proxy = self.auction_proxy(auction_contract_addr);
        if unstake_tokens {
            Ok(auction_proxy
                .unstake(bls_keys.into())
                .async_call()
                .with_callback(self.callbacks().auction_unstake_callback(node_ids)))
        } else {
            Ok(auction_proxy
                .unstake_nodes(bls_keys.into())
                .async_call()
                .with_callback(self.callbacks().auction_unstake_callback(node_ids)))
        }
    }

    /// Only finalize deactivation if we got confirmation from the auction contract.
    /// `#[callback]` also has be declared in lib.rs for the moment.
    #[callback]
    fn auction_unstake_callback(
        &self,
        node_ids: Vec<usize>, // #[callback_arg]
        #[call_result] call_result: AsyncCallResult<MultiResultVec<BLSStatusMultiArg>>,
    ) -> SCResult<()> {
        match call_result {
            AsyncCallResult::Ok(node_status_args) => {
                let (node_ids_ok, node_ids_fail) =
                    self.split_node_ids_by_err(node_ids, node_status_args);
                self.auction_unstake_callback_ok(node_ids_ok)?;
                self.auction_unstake_callback_fail(
                    node_ids_fail,
                    &b"unstaking failed for some nodes"[..],
                )?;
                Ok(())
            }
            AsyncCallResult::Err(error) => {
                self.auction_unstake_callback_fail(node_ids, error.err_msg.as_slice())
            }
        }
    }

    fn auction_unstake_callback_ok(&self, node_ids: Vec<usize>) -> SCResult<()> {
        if node_ids.is_empty() {
            return Ok(());
        }

        // set nodes to UnBondPeriod + save current block nonce
        let bl_nonce = self.blockchain().get_block_nonce();
        for &node_id in node_ids.iter() {
            self.set_node_state(node_id, NodeState::UnBondPeriod { started: bl_nonce });
        }

        // log event (no data)
        // TODO: log BLS keys of nodes in data
        self.unstake_node_ok_event(());

        Ok(())
    }

    fn auction_unstake_callback_fail(&self, node_ids: Vec<usize>, err_msg: &[u8]) -> SCResult<()> {
        if node_ids.is_empty() {
            return Ok(());
        }

        // revert nodes to Active
        for &node_id in node_ids.iter() {
            self.set_node_state(node_id, NodeState::Active);
        }

        // log failure event (no data)
        self.unstake_node_fail_event(err_msg);

        Ok(())
    }

    // UNBOND
    /// Calls unbond for all provided nodes. Will fail if node cannot be unbonded.
    #[endpoint(unBondNodes)]
    fn unbond_specific_nodes_endpoint(
        &self,
        #[var_args] bls_keys: VarArgs<BLSKey>,
    ) -> SCResult<AsyncCall<Self::SendApi>> {
        only_owner!(self, "only owner allowed to unbond nodes");

        require!(
            !self.is_global_op_in_progress(),
            "node operations are temporarily paused as checkpoint is reset"
        );

        require!(!bls_keys.is_empty(), "no BLS keys provided");

        let mut node_ids = Vec::<usize>::with_capacity(bls_keys.len());
        for bls_key in bls_keys.iter() {
            let node_id = self.get_node_id(bls_key);
            require!(node_id != 0, "unknown node provided");
            require!(
                self.prepare_node_for_unbond_if_possible(node_id),
                "node cannot be unbonded"
            );
            node_ids.push(node_id);
        }

        Ok(self.perform_unbond(node_ids, bls_keys.into_vec()))
    }

    /// Calls unbond for all nodes that are in the unbond period and are due.
    /// Nothing happens if no nodes can be unbonded.
    #[endpoint(unBondAllPossibleNodes)]
    fn unbond_all_possible_nodes(&self) -> SCResult<OptionalResult<AsyncCall<Self::SendApi>>> {
        only_owner!(self, "only owner allowed to unbond nodes");

        require!(
            !self.is_global_op_in_progress(),
            "node operations are temporarily paused as checkpoint is reset"
        );

        let mut node_id = self.num_nodes().get();
        let mut node_ids = Vec::<usize>::new();
        let mut bls_keys = Vec::<BLSKey>::new();
        while node_id >= 1 {
            if self.prepare_node_for_unbond_if_possible(node_id) {
                node_ids.push(node_id);
                bls_keys.push(self.get_node_id_to_bls(node_id));
            }

            node_id -= 1;
        }

        if node_ids.is_empty() {
            return Ok(OptionalResult::None);
        }

        Ok(OptionalResult::Some(
            self.perform_unbond(node_ids, bls_keys),
        ))
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
        node_ids: Vec<usize>,
        bls_keys: Vec<BLSKey>,
    ) -> AsyncCall<Self::SendApi> {
        // send unbond command to Auction SC
        let auction_contract_addr = self.get_auction_contract_address();
        self.auction_proxy(auction_contract_addr)
            .unbond_nodes(bls_keys.into())
            .async_call()
            .with_callback(self.callbacks().auction_unbond_callback(node_ids))
    }

    /// Only finalize deactivation if we got confirmation from the auction contract.
    /// `#[callback]` also has be declared in lib.rs for the moment.
    #[callback]
    fn auction_unbond_callback(
        &self,
        node_ids: Vec<usize>,
        #[call_result] call_result: AsyncCallResult<MultiResultVec<BLSStatusMultiArg>>,
    ) -> SCResult<()> {
        match call_result {
            AsyncCallResult::Ok(node_status_args) => {
                let (node_ids_ok, node_ids_fail) =
                    self.split_node_ids_by_err(node_ids, node_status_args);
                self.auction_unbond_callback_ok(node_ids_ok)?;
                self.auction_unbond_callback_fail(
                    node_ids_fail,
                    &b"unbonding failed for some nodes"[..],
                )?;
                Ok(())
            }
            AsyncCallResult::Err(error) => {
                self.auction_unbond_callback_fail(node_ids, error.err_msg.as_slice())
            }
        }
    }

    fn auction_unbond_callback_ok(&self, node_ids: Vec<usize>) -> SCResult<()> {
        if node_ids.is_empty() {
            return Ok(());
        }

        // set nodes to Inactive + reset unstake nonce since it is no longer needed
        for &node_id in node_ids.iter() {
            self.set_node_state(node_id, NodeState::Inactive);
        }

        // log event (no data)
        // TODO: log BLS keys of nodes in data
        self.unbond_node_ok_event(());

        Ok(())
    }

    fn auction_unbond_callback_fail(&self, node_ids: Vec<usize>, err_msg: &[u8]) -> SCResult<()> {
        if node_ids.is_empty() {
            return Ok(());
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
                return sc_error!("node not pending unbond");
            }
        }

        // log failure event (no data)
        self.unbond_node_fail_event(err_msg);

        Ok(())
    }

    /// Claims from auction SC funds that were sent but are not required to run the nodes.
    #[endpoint(claimUnusedFunds)]
    fn claim_unused_funds(&self) -> SCResult<AsyncCall<Self::SendApi>> {
        only_owner!(self, "only owner can claim inactive stake from auction");

        require!(
            !self.is_global_op_in_progress(),
            "node operations are temporarily paused as checkpoint is reset"
        );

        // send claim command to Auction SC
        let auction_contract_addr = self.get_auction_contract_address();
        Ok(self
            .auction_proxy(auction_contract_addr)
            .claim()
            .async_call())
    }

    #[payable("EGLD")]
    #[endpoint(unJailNodes)]
    fn unjail_nodes(
        &self,
        #[var_args] bls_keys: VarArgs<BLSKey>,
        #[payment] fine_payment: Self::BigUint,
    ) -> SCResult<AsyncCall<Self::SendApi>> {
        only_owner!(self, "only owner allowed to unjail nodes");

        // validation only
        for bls_key in bls_keys.iter() {
            let node_id = self.get_node_id(bls_key);
            require!(node_id != 0, "unknown node provided");
            require!(
                self.get_node_state(node_id) == NodeState::Active,
                "node must be active"
            );
        }

        // send unJail command to Auction SC
        let auction_contract_addr = self.get_auction_contract_address();
        Ok(self
            .auction_proxy(auction_contract_addr)
            .with_token_transfer(TokenIdentifier::egld(), fine_payment)
            .unjail(bls_keys)
            .async_call())
    }
}
