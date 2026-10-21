//! The individual operations a fuzz run performs, each also reachable from the CLI.
//!
//! These are the ports of the Go fuzzer's executor methods. Everything that talks to the
//! contract lives in [`crate::ops`]; what is here is the harness logic around it -
//! looping until an operation completes, forwarding proceeds to the sinks, and reporting.

use multiversx_sc_snippets::imports::*;

use crate::{
    deleg_interactor_stats::print_stake_by_type,
    display::{display_egld_amount, display_percentage},
    ops::{self, NODE_STATE_INACTIVE, OpResult, operation_completion_status},
    test_env::{TestInteractor, parse_amount},
};

impl TestInteractor {
    pub async fn deploy(&mut self) {
        let owner = self.config.owner.address();
        let new_address = ops::deploy(
            &mut self.interactor,
            &owner,
            &self.config.deploy,
            &self.config.gas,
        )
        .await;

        println!("New contract address: {new_address}");
        println!("Set it as sc_address in the config file to use it.");
    }

    // ----- delegator operations -----

    pub async fn stake(&mut self, amount: Option<String>, delegator: Option<usize>) {
        let user = self.sender(delegator).await;
        let amount = match amount {
            Some(raw) => parse_amount(&raw, "amount"),
            None => self.config.stake_amount(),
        };

        report("stake", self.stake_from(&user, &amount).await);
    }

    pub(crate) async fn stake_from(
        &mut self,
        user: &Bech32Address,
        amount: &RustBigUint,
    ) -> OpResult {
        println!(
            "stake, delegator: {user}, amount: {}",
            display_egld_amount(amount)
        );
        self.ops().stake(user, amount).await
    }

    pub async fn unstake(&mut self, amount: Option<String>, delegator: Option<usize>) {
        let user = self.sender(delegator).await;
        let amount = match amount {
            Some(raw) => parse_amount(&raw, "amount"),
            None => self.config.unstake_amount(),
        };

        report("unStake", self.unstake_from(&user, &amount).await);
    }

    pub(crate) async fn unstake_from(
        &mut self,
        user: &Bech32Address,
        amount: &RustBigUint,
    ) -> OpResult {
        println!(
            "unStake, delegator: {user}, amount: {}",
            display_egld_amount(amount)
        );
        self.ops().unstake(user, amount).await
    }

    /// Withdraws everything currently unbondable, then forwards it to the withdraw sink.
    ///
    /// The endpoint works in bounded chunks, so this keeps calling while something is
    /// still unbondable - with the proviso that a call which moves nothing ends the loop,
    /// since otherwise an endpoint that stops making progress would spin forever.
    pub async fn unbond(&mut self, delegator: Option<usize>) {
        let user = self.sender(delegator).await;
        report("unBond", self.unbond_from(&user).await);
    }

    /// Returns how much was unbonded, so a caller tracking totals does not have to read
    /// the sink to find out.
    pub(crate) async fn unbond_from(&mut self, user: &Bech32Address) -> OpResult<RustBigUint> {
        let withdraw_sink = self.accounts().withdraw_sink.clone();

        let mut total = RustBigUint::default();
        let mut call_count = 0u32;

        while self.ops().unbondable(user).await > RustBigUint::default() {
            let unbonded = self.ops().unbond_user(user).await?;
            call_count += 1;
            println!(
                "unBond call #{call_count}: {}",
                display_egld_amount(&unbonded)
            );

            if unbonded == RustBigUint::default() {
                println!("Call moved nothing while stake was still unbondable, stopping.");
                break;
            }
            total += unbonded;
        }

        println!("unBond total: {}", display_egld_amount(&total));
        self.forward(user, &withdraw_sink, &total).await;
        Ok(total)
    }

    /// Claims rewards and forwards them to the rewards sink.
    pub async fn claim_rewards(&mut self, delegator: Option<usize>) {
        let user = self.sender(delegator).await;
        report("claim", self.claim_from(&user).await);
    }

    /// Returns how much was claimed.
    pub(crate) async fn claim_from(&mut self, user: &Bech32Address) -> OpResult<RustBigUint> {
        let rewards_sink = self.accounts().rewards_sink.clone();

        let claimable = self.ops().claimable_rewards(user).await;
        if claimable == RustBigUint::default() {
            println!("no rewards, delegator: {user}");
            return Ok(claimable);
        }

        self.ops().claim_rewards(user).await?;

        let remaining = self.ops().claimable_rewards(user).await;
        assert_eq!(
            remaining,
            RustBigUint::default(),
            "getClaimableRewards should always be zero after claimRewards, got {remaining}"
        );

        println!(
            "claim rewards, delegator: {user}, amount: {}",
            display_egld_amount(&claimable)
        );
        self.forward(user, &rewards_sink, &claimable).await;
        Ok(claimable)
    }

    pub async fn delegate_vote(&mut self, delegator: Option<usize>) {
        let user = self.sender(delegator).await;
        let proposal = self.config.actions.vote_proposal;
        let decision = self.config.actions.vote_decision.clone();

        report(
            "vote",
            self.ops().delegate_vote(&user, proposal, &decision).await,
        );
    }

    // ----- owner operations -----

    pub async fn add_nodes_action(&mut self, count: usize) {
        report("addNodes", self.add_nodes(count).await);
    }

    pub async fn set_service_fee_action(&mut self, service_fee_per_10000: usize) {
        report(
            "setServiceFee",
            self.set_service_fee(service_fee_per_10000).await,
        );
    }

    pub(crate) async fn add_nodes(&mut self, count: usize) -> OpResult {
        let owner = self.config.owner.address();
        let first_index = self.ops().num_nodes().await;

        self.ops().add_nodes(&owner, first_index, count).await?;

        println!(
            "Added {count} node(s), key indexes {first_index}..{}",
            first_index + count
        );
        Ok(())
    }

    /// Removes the most recently added nodes that can still be removed.
    ///
    /// Only inactive nodes qualify, and removal marks a node rather than deleting it, so
    /// the candidates are picked from the node states rather than from the node count -
    /// which never goes down.
    pub async fn remove_nodes(&mut self, count: usize) {
        let owner = self.config.owner.address();

        let removable: Vec<usize> = self
            .ops()
            .node_states()
            .await
            .iter()
            .enumerate()
            .filter(|(_, state)| **state == NODE_STATE_INACTIVE)
            .map(|(index, _)| index)
            .rev()
            .take(count)
            .collect();

        if removable.is_empty() {
            println!("No inactive nodes to remove.");
            return;
        }

        match self.ops().remove_nodes(&owner, &removable).await {
            Ok(()) => println!(
                "Removed {} node(s), key indexes {removable:?}",
                removable.len()
            ),
            Err(message) => println!("Could not remove nodes: {message}"),
        }
    }

    pub(crate) async fn set_service_fee(&mut self, service_fee_per_10000: usize) -> OpResult {
        let owner = self.config.owner.address();

        let status = self
            .ops()
            .set_service_fee(&owner, service_fee_per_10000)
            .await?;

        println!(
            "Service fee set to {}: {}",
            display_percentage(service_fee_per_10000 as u32),
            operation_completion_status(status)
        );
        Ok(())
    }

    pub async fn modify_delegation_cap(&mut self, amount: Option<String>) {
        let new_cap = match amount {
            Some(raw) => parse_amount(&raw, "amount"),
            None => self.config.delegation_cap(),
        };

        report(
            "modify delegation cap",
            self.modify_delegation_cap_to(&new_cap).await,
        );
    }

    pub(crate) async fn modify_delegation_cap_to(&mut self, new_cap: &RustBigUint) -> OpResult {
        let owner = self.config.owner.address();

        let status = self
            .ops()
            .modify_total_delegation_cap(&owner, new_cap)
            .await?;

        println!(
            "modify delegation cap: {}, new cap {}",
            operation_completion_status(status),
            display_egld_amount(new_cap)
        );
        Ok(())
    }

    /// Drives the current global operation to completion. Changing the delegation cap or
    /// the service fee may leave one in progress, and most endpoints are blocked until it
    /// finishes.
    pub async fn continue_global_operation(&mut self) {
        let owner = self.config.owner.address();

        match self.ops().continue_global_operation(&owner).await {
            Some(call_count) => println!("Global operation completed after {call_count} call(s)."),
            None => println!("No global operation in progress."),
        }
    }

    /// Sweeps fund entries smaller than the limit, waiting list first, then active.
    pub async fn dust_cleanup(&mut self, limit: Option<String>) {
        let limit = match limit {
            Some(raw) => parse_amount(&raw, "limit"),
            None => self.config.dust_limit(),
        };

        report("dust cleanup", self.dust_cleanup_below(&limit).await);
    }

    pub(crate) async fn dust_cleanup_below(&mut self, limit: &RustBigUint) -> OpResult {
        let owner = self.config.owner.address();

        println!("dust cleanup, limit: {}", display_egld_amount(limit));

        let mut remaining = self.ops().count_dust_items_waiting_list(limit).await;
        println!("Dust items in the waiting list: {remaining}");
        while remaining > 0 {
            self.ops().dust_cleanup_waiting_list(&owner, limit).await?;

            let left = self.ops().count_dust_items_waiting_list(limit).await;
            println!("  -> {left} left");
            if left >= remaining {
                println!("  cleanup made no progress, stopping.");
                break;
            }
            remaining = left;
        }

        let mut remaining = self.ops().count_dust_items_active(limit).await;
        println!("Dust items in the active list: {remaining}");
        while remaining > 0 {
            self.ops().dust_cleanup_active(&owner, limit).await?;

            let left = self.ops().count_dust_items_active(limit).await;
            println!("  -> {left} left");
            if left >= remaining {
                println!("  cleanup made no progress, stopping.");
                break;
            }
            remaining = left;
        }

        Ok(())
    }

    // ----- harness operations -----

    /// Simulates a validator reward arriving at the contract.
    ///
    /// Chain simulator only: writes the protocol's own cumulated-reward storage key, the
    /// same one a real reward transaction writes.
    pub async fn add_rewards(&mut self, amount: Option<String>) {
        let amount = match amount {
            Some(raw) => parse_amount(&raw, "amount"),
            None => self.config.reward_amount(),
        };

        self.inject_reward(&amount).await;
    }

    pub(crate) async fn inject_reward(&mut self, amount: &RustBigUint) {
        let sc_address = self.config.chain_config.sc_address().clone();

        let before = self.ops().total_cumulated_rewards().await;
        let balance_before = self.balance(&sc_address).await;

        self.interactor
            .inject_validator_reward(&sc_address, amount)
            .await
            .expect("could not inject a validator reward");

        let after = self.ops().total_cumulated_rewards().await;
        let balance_after = self.balance(&sc_address).await;

        println!("Injected reward:          {}", display_egld_amount(amount));
        println!("Cumulated rewards before: {}", display_egld_amount(&before));
        println!("Cumulated rewards after:  {}", display_egld_amount(&after));
        println!(
            "Contract balance:         {} -> {}",
            display_egld_amount(&balance_before),
            display_egld_amount(&balance_after)
        );

        let observed = &after - &before;
        assert_eq!(
            &observed, amount,
            "contract saw a different reward than was injected"
        );
    }

    /// Advances the chain. Asking for no blocks is a no-op rather than an error - the
    /// simulator refuses a zero-block request, and every caller here would rather carry on.
    pub async fn generate_blocks(&mut self, num_blocks: u64) {
        if num_blocks == 0 {
            println!("No blocks to generate.");
            return;
        }

        self.interactor
            .generate_blocks(num_blocks)
            .await
            .expect("could not generate blocks");

        println!("Generated {num_blocks} block(s).");
    }

    /// Runs the contract's own consistency checks, the ones the Go fuzzer ran after most
    /// events. Both are endpoints that fail when the property is broken.
    pub async fn check_invariants(&mut self) {
        let failures = self.invariant_failures().await;
        if failures.is_empty() {
            println!("All invariants hold.");
            return;
        }

        for failure in failures {
            println!("BROKEN - {failure}");
        }
    }

    /// Runs the contract's own consistency checks and returns what broke, if anything.
    pub(crate) async fn invariant_failures(&mut self) -> Vec<String> {
        let owner = self.config.owner.address();

        [
            (
                "owner stake share",
                self.ops().validate_owner_stake_share(&owner).await,
            ),
            (
                "delegation cap",
                self.ops().validate_delegation_cap_invariant(&owner).await,
            ),
        ]
        .into_iter()
        .filter_map(|(name, result)| result.err().map(|error| format!("{name}: {error}")))
        .collect()
    }

    /// A quick look at the contract, without needing the real-chain config.
    pub async fn status(&mut self) {
        let stake_by_type = self.ops().total_stake_by_type().await;
        let cap = self.ops().total_delegation_cap().await;
        let service_fee = self.ops().service_fee().await;
        let rewards = self.ops().total_cumulated_rewards().await;
        let num_users = self.ops().num_users().await;
        let num_nodes = self.ops().num_nodes().await;

        print_stake_by_type(&stake_by_type);
        println!();
        println!("Delegation cap:    {}", display_egld_amount(&cap));
        println!("Cumulated rewards: {}", display_egld_amount(&rewards));
        println!("Service fee:       {}", display_percentage(service_fee));
        println!("Users:             {num_users}");
        println!("Nodes:             {num_nodes}");

        let accounts = self.accounts();
        let rewards_sink = self.balance(&accounts.rewards_sink).await;
        let withdraw_sink = self.balance(&accounts.withdraw_sink).await;

        println!();
        println!("Rewards sink:      {}", display_egld_amount(&rewards_sink));
        println!("Withdraw sink:     {}", display_egld_amount(&withdraw_sink));
    }
}

/// Prints a refusal rather than letting it abort a hand-run command.
fn report<T>(what: &str, result: OpResult<T>) {
    if let Err(error) = result {
        println!("{what} failed: {error}");
    }
}
