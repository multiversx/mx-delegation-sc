//! The run loop: draw an event, execute it, check the contract's invariants, repeat.

use std::time::{SystemTime, UNIX_EPOCH};

use multiversx_sc_snippets::imports::*;
use rand::{RngExt, SeedableRng, rngs::StdRng};

use crate::{
    deleg_interactor_config::DeployConfig,
    deleg_interactor_stats::print_stake_by_type,
    display::{display_egld_amount, display_percentage},
    fuzz::{Event, bounds::Bounds, log::ActionLog, roulette, state::FuzzState},
    test_env::{TestAccounts, TestInteractor},
};

pub struct FuzzRunner {
    interact: TestInteractor,
    accounts: TestAccounts,
    rng: StdRng,
    bounds: Bounds,
    state: FuzzState,
    log: ActionLog,
    seed: u64,
    iterations: usize,
    invariant_check_every: usize,
}

impl FuzzRunner {
    /// Deploys a contract with randomized settings, prepares the accounts, and returns a
    /// runner ready to step.
    pub async fn start(
        config_file: &str,
        seed_override: Option<u64>,
        iterations_override: Option<usize>,
    ) -> Self {
        let mut interact = TestInteractor::init(config_file).await;

        let seed = seed_override
            .or_else(|| Some(interact.config.fuzz.seed).filter(|seed| *seed != 0))
            .unwrap_or_else(clock_seed);
        let iterations = iterations_override.unwrap_or(interact.config.fuzz.iterations);
        let invariant_check_every = interact.config.fuzz.invariant_check_every.max(1);

        println!("Random seed: {seed}");
        println!("Iterations:  {iterations}");

        let mut rng = StdRng::seed_from_u64(seed);
        let bounds = Bounds::from_config(&interact.config);
        let log_path = interact
            .interactor
            .current_dir
            .join(&interact.config.fuzz.log_file);
        let log = ActionLog::create(&log_path, seed, iterations);
        println!("Action log:  {}", log_path.display());

        let deploy = randomized_deploy(&mut rng, &interact.config);
        println!(
            "Deploying with service fee {}, cap {}, unbond after {} block(s)",
            display_percentage(deploy.service_fee_per_10000 as u32),
            display_egld_amount(&deploy.total_delegation_cap()),
            deploy.n_blocks_before_unbond,
        );

        let (sc_address, accounts) = interact.setup(&deploy).await;
        interact.report_setup(&sc_address, &accounts).await;

        FuzzRunner {
            interact,
            accounts,
            rng,
            bounds,
            state: FuzzState::default(),
            log,
            seed,
            iterations,
            invariant_check_every,
        }
    }

    pub async fn run(&mut self) {
        // The Go fuzzer let some blocks pass before the first event, so deferred payments
        // do not all share a registration block.
        let initial_advance = self
            .rng
            .random_range(1..=self.bounds.block_advance_max.max(1));
        self.interact.generate_blocks(initial_advance).await;

        for step in 1..=self.iterations {
            let event = Event::random(
                &mut self.rng,
                &self.interact.config.fuzz.weights,
                &self.bounds,
            );
            self.log.write(step, &event);

            println!();
            println!("--- step {step}/{} : {event} ---", self.iterations);
            self.execute(step, event.clone()).await;

            if event.checks_invariants() && step % self.invariant_check_every == 0 {
                self.check_invariants(step).await;
            }
        }

        self.report().await;
    }

    async fn execute(&mut self, step: usize, event: Event) {
        match event {
            Event::AdvanceBlocks(blocks) => self.interact.generate_blocks(blocks).await,
            Event::AddNodes(count) => self.add_nodes(step, count).await,

            // The contract refuses unless every chosen node is inactive, which it reports
            // for itself.
            Event::RemoveNodes(count) => self.interact.remove_nodes(count).await,

            Event::Stake { delegator, amount } => self.stake(step, delegator, &amount).await,
            Event::AddRewards(amount) => self.add_rewards(&amount).await,
            Event::ClaimRewards { delegator } => self.claim(step, delegator).await,
            Event::Unstake { delegator, amount } => self.unstake(step, delegator, &amount).await,
            Event::Unbond { delegator } => self.unbond(step, delegator).await,

            Event::ModifyCap(cap) => {
                let result = self.interact.modify_delegation_cap_to(&cap).await;
                if self
                    .tolerate(step, "modify delegation cap", result)
                    .is_some()
                {
                    self.interact.continue_global_operation().await;
                }
            }
            Event::SetServiceFee(fee) => {
                let result = self.interact.set_service_fee(fee).await;
                if self.tolerate(step, "set service fee", result).is_some() {
                    self.interact.continue_global_operation().await;
                }
            }

            Event::DustCleanup(limit) => {
                let result = self.interact.dust_cleanup_below(&limit).await;
                self.require(step, "dust cleanup", result);
            }
        }
    }

    async fn add_nodes(&mut self, step: usize, count: usize) {
        if count == 0 {
            return;
        }

        let result = self.interact.add_nodes(count).await;
        self.require(step, "addNodes", result);
        self.state.nodes_added += count;
    }

    /// Staking is refused below the minimum stake, or once the delegation cap is full.
    /// Only a stake that went through counts towards the total.
    async fn stake(&mut self, step: usize, delegator: usize, amount: &RustBigUint) {
        let user = self.accounts.delegator(delegator).clone();

        let result = self.interact.stake_from(&user, amount).await;
        if self.tolerate(step, "stake", result).is_some() {
            self.state.total_stake_added += amount;
        }
    }

    /// Refused for an address that never staked, or for more than it holds - both of
    /// which the random draw produces constantly.
    async fn unstake(&mut self, step: usize, delegator: usize, amount: &RustBigUint) {
        let user = self.accounts.delegator(delegator).clone();

        let result = self.interact.unstake_from(&user, amount).await;
        self.tolerate(step, "unStake", result);
    }

    /// Rewards only accrue once the contract is past bootstrap mode, which is where the
    /// Go fuzzer skipped this event too.
    async fn add_rewards(&mut self, amount: &RustBigUint) {
        if self.interact.ops().is_bootstrap_mode().await {
            println!("still in bootstrap mode, no rewards");
            return;
        }

        self.interact.inject_reward(amount).await;
        self.state.total_rewards += amount;
    }

    async fn claim(&mut self, step: usize, delegator: usize) {
        let user = self.accounts.delegator(delegator).clone();

        let result = self.interact.claim_from(&user).await;
        self.state.total_claimed += self.require(step, "claimRewards", result);
    }

    async fn unbond(&mut self, step: usize, delegator: usize) {
        let user = self.accounts.delegator(delegator).clone();

        let result = self.interact.unbond_from(&user).await;
        self.state.total_withdrawn += self.require(step, "unBond", result);
    }

    /// A refusal the contract makes as a matter of course. The Go fuzzer logged these and
    /// carried on; running out of gas is the one case it treated as a defect, because it
    /// means the endpoint cannot finish within its budget.
    fn tolerate<T>(
        &self,
        step: usize,
        what: &str,
        result: Result<T, TxResponseStatus>,
    ) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(error) if error.status == ReturnCode::OutOfGas => self.fail(step, what, &error),
            Err(error) => {
                println!("{what} refused: {}", error.message);
                None
            }
        }
    }

    /// An operation that should have gone through. Anything else ends the run.
    fn require<T>(&self, step: usize, what: &str, result: Result<T, TxResponseStatus>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => self.fail(step, what, &error),
        }
    }

    fn fail(&self, step: usize, what: &str, error: &TxResponseStatus) -> ! {
        println!();
        println!("UNEXPECTED FAILURE at step {step}, seed {}", self.seed);
        println!("  {what}: {}", error.message);
        panic!("{what} failed at step {step}");
    }

    async fn check_invariants(&mut self, step: usize) {
        let failures = self.interact.invariant_failures().await;
        if failures.is_empty() {
            return;
        }

        println!();
        println!("INVARIANT BROKEN at step {step}, seed {}:", self.seed);
        for failure in &failures {
            println!("  {failure}");
        }
        panic!("invariant broken at step {step}");
    }

    async fn report(&mut self) {
        let stake_by_type = self.interact.ops().total_stake_by_type().await;
        let cumulated_rewards = self.interact.ops().total_cumulated_rewards().await;
        let rewards_sink = self.interact.balance(&self.accounts.rewards_sink).await;
        let withdraw_sink = self.interact.balance(&self.accounts.withdraw_sink).await;

        println!();
        println!(
            "=== run finished: {} events, seed {} ===",
            self.iterations, self.seed
        );
        println!();
        print_stake_by_type(&stake_by_type);
        println!();
        println!(
            "Stake added:       {}",
            display_egld_amount(&self.state.total_stake_added)
        );
        println!(
            "Rewards injected:  {}",
            display_egld_amount(&self.state.total_rewards)
        );
        println!(
            "Contract rewards:  {}",
            display_egld_amount(&cumulated_rewards)
        );
        println!(
            "Rewards claimed:   {}",
            display_egld_amount(&self.state.total_claimed)
        );
        println!(
            "Stake withdrawn:   {}",
            display_egld_amount(&self.state.total_withdrawn)
        );
        println!("Nodes added:       {}", self.state.nodes_added);
        println!();
        println!("Rewards sink:      {}", display_egld_amount(&rewards_sink));
        println!("Withdraw sink:     {}", display_egld_amount(&withdraw_sink));

        assert_eq!(
            self.state.total_rewards, cumulated_rewards,
            "the contract's cumulated rewards should match what was injected"
        );
        assert_eq!(
            self.state.total_claimed, rewards_sink,
            "the rewards sink should hold exactly what was claimed"
        );
        assert_eq!(
            self.state.total_withdrawn, withdraw_sink,
            "the withdraw sink should hold exactly what was unbonded"
        );
    }
}

/// Constructor arguments drawn at random, the way the Go fuzzer's `init` did.
///
/// The code path and the auction address come from the configured template; everything
/// else varies per run, so a campaign covers different settings rather than one.
fn randomized_deploy(rng: &mut StdRng, config: &crate::test_env::TestConfig) -> DeployConfig {
    let fuzz = &config.fuzz;

    DeployConfig {
        service_fee_per_10000: rng.random_range(0..fuzz.max_service_fee),
        owner_min_stake_share_per_10000: 0,
        n_blocks_before_unbond: rng.random_range(0..=fuzz.max_n_blocks_before_unbond),
        minimum_stake: roulette::below(rng, &fuzz.max_minimum_stake()).to_string(),
        total_delegation_cap: roulette::below(rng, &fuzz.max_delegation_cap()).to_string(),
        ..config.deploy.clone()
    }
}

fn clock_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before the epoch")
        .as_nanos() as u64
}
