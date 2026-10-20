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
            vote_proposal: 1,
            vote_decision: "yes".to_owned(),
        }
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
}

impl TestConfig {
    pub fn stake_amount(&self) -> BigUint<StaticApi> {
        parse_amount(&self.actions.stake_amount, "stake_amount")
    }

    pub fn unstake_amount(&self) -> BigUint<StaticApi> {
        parse_amount(&self.actions.unstake_amount, "unstake_amount")
    }

    pub fn delegation_cap(&self) -> BigUint<StaticApi> {
        parse_amount(&self.actions.delegation_cap, "delegation_cap")
    }

    pub fn reward_amount(&self) -> RustBigUint {
        self.actions
            .reward_amount
            .parse()
            .unwrap_or_else(|_| panic!("invalid reward_amount: '{}'", self.actions.reward_amount))
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
pub fn parse_amount(raw: &str, field: &str) -> BigUint<StaticApi> {
    let parsed: RustBigUint = raw
        .parse()
        .unwrap_or_else(|_| panic!("invalid {field}: '{raw}'"));
    BigUint::<StaticApi>::from(parsed)
}
