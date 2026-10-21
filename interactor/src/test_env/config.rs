use multiversx_sc_snippets::imports::*;
use serde::Deserialize;

use crate::deleg_interactor_config::{ChainConfig, DeployConfig, GasConfig};

/// Default amounts for the manual test actions, so the CLI can be invoked without
/// spelling out a value every time.
///
/// All amounts are in atomic units (1 EGLD = 10^18), as strings, because they routinely
/// exceed what TOML integers hold.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ActionsConfig {
    pub stake_amount: String,
    pub unstake_amount: String,
    pub reward_amount: String,
    pub delegation_cap: String,
    pub dust_limit: String,
    pub vote_proposal: u64,
    pub vote_decision: String,
}

impl Default for ActionsConfig {
    fn default() -> Self {
        ActionsConfig {
            stake_amount: "2000000000000000000".to_owned(),
            unstake_amount: "1000000000000000000".to_owned(),
            reward_amount: "1000000000000000000".to_owned(),
            delegation_cap: "100000000000000000000000".to_owned(),
            dust_limit: "100000000000000000".to_owned(),
            vote_proposal: 1,
            vote_decision: "yes".to_owned(),
        }
    }
}

/// The delegators a fuzz run operates with.
#[derive(Debug, Deserialize)]
pub struct DelegatorsConfig {
    /// How many delegators to generate, on top of the owner.
    pub count: usize,

    /// Seed the addresses are derived from. The same seed always yields the same
    /// addresses, so a run needs nothing stored on disk to be repeatable.
    pub seed: u64,

    /// Starting balance for each generated delegator, in atomic units, as a string.
    pub initial_balance: String,
}

impl DelegatorsConfig {
    pub fn initial_balance(&self) -> RustBigUint {
        parse_amount(&self.initial_balance, "initial_balance")
    }
}

/// How often each kind of event is drawn.
///
/// The defaults are the Go fuzzer's roulette weights. `unbond` is 10 rather than 5
/// because the Go table listed the same unBond outcome twice.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct EventWeights {
    pub advance_blocks: u32,
    pub add_nodes: u32,
    pub remove_nodes: u32,
    pub stake: u32,
    pub add_rewards: u32,
    pub claim_rewards: u32,
    pub unstake: u32,
    pub unbond: u32,
    pub modify_cap: u32,
    pub set_service_fee: u32,
    pub dust_cleanup: u32,
}

impl Default for EventWeights {
    fn default() -> Self {
        EventWeights {
            advance_blocks: 5,
            add_nodes: 5,
            remove_nodes: 5,
            stake: 5,
            add_rewards: 5,
            claim_rewards: 20,
            unstake: 5,
            unbond: 10,
            modify_cap: 5,
            set_service_fee: 5,
            dust_cleanup: 1,
        }
    }
}

/// The fuzzer's parameter space.
///
/// The bounds mirror the Go fuzzer's, scaled from its raw units to EGLD, except for the
/// block counts: the Go values went up to 1000 per event, which at 20 rounds per epoch
/// would mean tens of epoch transitions for a single step.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct FuzzConfig {
    /// Random seed. Zero means derive one from the clock and print it.
    pub seed: u64,

    /// How many random events to generate.
    pub iterations: usize,

    /// Check the contract's invariants once every this many events. 1 matches the Go
    /// fuzzer, which checked after almost every step.
    pub invariant_check_every: usize,

    /// Upper bound for a single block advance.
    pub block_advance_max: u64,

    /// Upper bounds for the randomized amounts, in atomic units, as strings.
    pub max_stake: String,
    pub max_reward: String,
    pub max_delegation_cap: String,
    pub max_dust: String,
    pub max_minimum_stake: String,

    /// Upper bound for the service fee and the initial one, in 1/10000.
    pub max_service_fee: usize,

    /// Upper bound for the deployed contract's unbond period, in blocks.
    pub max_n_blocks_before_unbond: u64,

    /// Upper bounds for how many nodes one event adds or removes.
    pub max_nodes_added: usize,
    pub max_nodes_removed: usize,

    /// Where the per-event log is written, relative to the interactor directory.
    pub log_file: String,

    pub weights: EventWeights,
}

impl Default for FuzzConfig {
    fn default() -> Self {
        FuzzConfig {
            seed: 0,
            iterations: 1000,
            invariant_check_every: 1,
            block_advance_max: 20,
            max_stake: "2000000000000000000".to_owned(),
            max_reward: "1000000000000000000".to_owned(),
            max_delegation_cap: "4000000000000000000".to_owned(),
            max_dust: "250000000000000000".to_owned(),
            max_minimum_stake: "1000000000000000000".to_owned(),
            max_service_fee: 10000,
            max_n_blocks_before_unbond: 20,
            max_nodes_added: 3,
            max_nodes_removed: 2,
            log_file: "fuzz_run.jsonl".to_owned(),
            weights: EventWeights::default(),
        }
    }
}

impl FuzzConfig {
    pub fn max_stake(&self) -> RustBigUint {
        parse_amount(&self.max_stake, "max_stake")
    }

    pub fn max_reward(&self) -> RustBigUint {
        parse_amount(&self.max_reward, "max_reward")
    }

    pub fn max_delegation_cap(&self) -> RustBigUint {
        parse_amount(&self.max_delegation_cap, "max_delegation_cap")
    }

    pub fn max_dust(&self) -> RustBigUint {
        parse_amount(&self.max_dust, "max_dust")
    }

    pub fn max_minimum_stake(&self) -> RustBigUint {
        parse_amount(&self.max_minimum_stake, "max_minimum_stake")
    }
}

/// Configuration for work on the chain simulator: the fuzzer, and the manual test
/// actions that mirror the fuzzer's operations.
///
/// Loaded from `test.toml`. For investigating contracts on real chains see
/// [`crate::deleg_interactor_config::Config`], loaded from `config.toml`.
///
/// The two are structurally compatible on purpose, so `--config` can point either
/// command group at either file.
#[derive(Debug, Deserialize)]
pub struct TestConfig {
    pub chain_config: ChainConfig,
    pub deploy: DeployConfig,

    /// Gas limits. Optional, defaults apply per operation.
    #[serde(default)]
    pub gas: GasConfig,

    /// Defaults for the manual test actions. Optional.
    #[serde(default)]
    pub actions: ActionsConfig,

    /// Contract owner: deploys, upgrades, and the admin endpoints.
    pub owner: WalletConfig,

    /// Acting user: stake, unstake, claim, vote.
    pub user: WalletConfig,

    /// The delegators a fuzz run operates with.
    pub delegators: DelegatorsConfig,

    /// The fuzzer's parameter space. Optional - the manual actions do not need it.
    #[serde(default)]
    pub fuzz: FuzzConfig,
}

impl TestConfig {
    pub fn stake_amount(&self) -> RustBigUint {
        parse_amount(&self.actions.stake_amount, "stake_amount")
    }

    pub fn unstake_amount(&self) -> RustBigUint {
        parse_amount(&self.actions.unstake_amount, "unstake_amount")
    }

    pub fn delegation_cap(&self) -> RustBigUint {
        parse_amount(&self.actions.delegation_cap, "delegation_cap")
    }

    pub fn reward_amount(&self) -> RustBigUint {
        parse_amount(&self.actions.reward_amount, "reward_amount")
    }

    pub fn dust_limit(&self) -> RustBigUint {
        parse_amount(&self.actions.dust_limit, "dust_limit")
    }
}

impl InteractorConfig for TestConfig {
    fn connection(&self) -> &ConnectionConfig {
        &self.chain_config.connection
    }

    fn register_wallets(&self) -> Vec<Wallet> {
        vec![self.owner.wallet().clone(), self.user.wallet().clone()]
    }
}

/// Parses an atomic-unit amount from a config string.
pub fn parse_amount(raw: &str, field: &str) -> RustBigUint {
    raw.parse()
        .unwrap_or_else(|_| panic!("invalid {field}: '{raw}'"))
}
