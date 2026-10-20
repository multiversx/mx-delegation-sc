//! The individual operations a fuzz run performs, each also reachable from the CLI.
//!
//! These are the ports of the Go fuzzer's executor methods. Everything that talks to the
//! contract lives in [`crate::ops`]; what is here is the harness logic around it -
//! looping until an operation completes, forwarding proceeds to the sinks, and reporting.

use multiversx_sc_snippets::imports::*;

use crate::{
    deleg_interactor_stats::print_stake_by_type,
    display::{display_egld_amount, display_percentage},
    ops::{self, NODE_STATE_INACTIVE, operation_completion_status},
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

        println!("Staking {} from {user}", display_egld_amount(&amount));
        self.ops().stake(&user, &amount).await;
        println!("Stake done");
    }

    pub async fn unstake(&mut self, amount: Option<String>, delegator: Option<usize>) {
        let user = self.sender(delegator).await;
        let amount = match amount {
            Some(raw) => parse_amount(&raw, "amount"),
            None => self.config.unstake_amount(),
        };

        println!("Unstaking {} from {user}", display_egld_amount(&amount));
        self.ops().unstake(&user, &amount).await;
        println!("Unstake done");
    }

    /// Withdraws everything currently unbondable, then forwards it to the withdraw sink.
    ///
    /// The endpoint works in bounded chunks, so this keeps calling while something is
    /// still unbondable - with the proviso that a call which moves nothing ends the loop,
    /// since otherwise an endpoint that stops making progress would spin forever.
    pub async fn unbond(&mut self, delegator: Option<usize>) {
        let user = self.sender(delegator).await;
        let withdraw_sink = self.accounts().withdraw_sink.clone();

        let mut total = RustBigUint::default();
        let mut call_count = 0u32;

        while self.ops().unbondable(&user).await > RustBigUint::default() {
            let unbonded = self.ops().unbond_user(&user).await;
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

        println!("Unbonded total: {}", display_egld_amount(&total));
        self.forward(&user, &withdraw_sink, &total).await;
        if total > RustBigUint::default() {
            println!("Forwarded to the withdraw sink.");
        }
    }

    /// Claims rewards and forwards them to the rewards sink.
    pub async fn claim_rewards(&mut self, delegator: Option<usize>) {
        let user = self.sender(delegator).await;
        let rewards_sink = self.accounts().rewards_sink.clone();

        let claimable = self.ops().claimable_rewards(&user).await;
        if claimable == RustBigUint::default() {
            println!("No rewards to claim for {user}");
            return;
        }

        println!("Claimable: {}", display_egld_amount(&claimable));
        self.ops().claim_rewards(&user).await;

        let remaining = self.ops().claimable_rewards(&user).await;
        assert_eq!(
            remaining,
            RustBigUint::default(),
            "getClaimableRewards should always be zero after claimRewards, got {remaining}"
        );

        self.forward(&user, &rewards_sink, &claimable).await;
        println!("Claimed and forwarded to the rewards sink.");
    }

    pub async fn delegate_vote(&mut self, delegator: Option<usize>) {
        let user = self.sender(delegator).await;
        let proposal = self.config.actions.vote_proposal;
        let decision = self.config.actions.vote_decision.clone();

        self.ops().delegate_vote(&user, proposal, &decision).await;
        println!("Vote sent");
    }

    // ----- owner operations -----

    pub async fn add_nodes(&mut self, count: usize) {
        let owner = self.config.owner.address();
        let first_index = self.ops().num_nodes().await;

        self.ops().add_nodes(&owner, first_index, count).await;

        println!(
            "Added {count} node(s), key indexes {first_index}..{}",
            first_index + count
        );
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

    pub async fn set_service_fee(&mut self, service_fee_per_10000: usize) {
        let owner = self.config.owner.address();

        let status = self
            .ops()
            .set_service_fee(&owner, service_fee_per_10000)
            .await;

        println!(
            "Service fee set to {}: {}",
            display_percentage(service_fee_per_10000 as u32),
            operation_completion_status(status)
        );
    }

    pub async fn modify_delegation_cap(&mut self, amount: Option<String>) {
        let owner = self.config.owner.address();
        let new_cap = match amount {
            Some(raw) => parse_amount(&raw, "amount"),
            None => self.config.delegation_cap(),
        };

        println!("New delegation cap: {}", display_egld_amount(&new_cap));
        let status = self
            .ops()
            .modify_total_delegation_cap(&owner, &new_cap)
            .await;

        println!(
            "Modified delegation cap: {}",
            operation_completion_status(status)
        );
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
        let owner = self.config.owner.address();
        let limit = match limit {
            Some(raw) => parse_amount(&raw, "limit"),
            None => self.config.dust_limit(),
        };

        println!("Dust limit: {}", display_egld_amount(&limit));

        let mut remaining = self.ops().count_dust_items_waiting_list(&limit).await;
        println!("Dust items in the waiting list: {remaining}");
        while remaining > 0 {
            self.ops().dust_cleanup_waiting_list(&owner, &limit).await;

            let left = self.ops().count_dust_items_waiting_list(&limit).await;
            println!("  -> {left} left");
            if left >= remaining {
                println!("  cleanup made no progress, stopping.");
                break;
            }
            remaining = left;
        }

        let mut remaining = self.ops().count_dust_items_active(&limit).await;
        println!("Dust items in the active list: {remaining}");
        while remaining > 0 {
            self.ops().dust_cleanup_active(&owner, &limit).await;

            let left = self.ops().count_dust_items_active(&limit).await;
            println!("  -> {left} left");
            if left >= remaining {
                println!("  cleanup made no progress, stopping.");
                break;
            }
            remaining = left;
        }
    }

    // ----- harness operations -----

    /// Simulates a validator reward arriving at the contract.
    ///
    /// Chain simulator only: writes the protocol's own cumulated-reward storage key, the
    /// same one a real reward transaction writes.
    pub async fn add_rewards(&mut self, amount: Option<String>) {
        let sc_address = self.config.chain_config.sc_address().clone();
        let amount = match amount {
            Some(raw) => parse_amount(&raw, "amount"),
            None => self.config.reward_amount(),
        };

        let before = self.ops().total_cumulated_rewards().await;
        let balance_before = self.balance(&sc_address).await;

        self.interactor
            .inject_validator_reward(&sc_address, &amount)
            .await
            .expect("could not inject a validator reward");

        let after = self.ops().total_cumulated_rewards().await;
        let balance_after = self.balance(&sc_address).await;

        println!("Injected reward:          {}", display_egld_amount(&amount));
        println!("Cumulated rewards before: {}", display_egld_amount(&before));
        println!("Cumulated rewards after:  {}", display_egld_amount(&after));
        println!(
            "Contract balance:         {} -> {}",
            display_egld_amount(&balance_before),
            display_egld_amount(&balance_after)
        );

        let observed = &after - &before;
        if observed == amount {
            println!("Contract sees the full reward.");
        } else {
            println!(
                "MISMATCH: contract saw {}, expected {}",
                display_egld_amount(&observed),
                display_egld_amount(&amount)
            );
        }
    }

    pub async fn generate_blocks(&mut self, num_blocks: u64) {
        self.interactor
            .generate_blocks(num_blocks)
            .await
            .expect("could not generate blocks");

        println!("Generated {num_blocks} block(s).");
    }

    /// Runs the contract's own consistency checks, the ones the Go fuzzer ran after most
    /// events. Both are endpoints that fail when the property is broken.
    pub async fn check_invariants(&mut self) {
        let owner = self.config.owner.address();
        let mut all_passed = true;

        for (name, result) in [
            (
                "owner stake share",
                self.ops().validate_owner_stake_share(&owner).await,
            ),
            (
                "delegation cap",
                self.ops().validate_delegation_cap_invariant(&owner).await,
            ),
        ] {
            match result {
                Ok(()) => println!("{name}: ok"),
                Err(message) => {
                    all_passed = false;
                    println!("{name}: BROKEN - {message}");
                },
            }
        }

        if all_passed {
            println!("All invariants hold.");
        }
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
