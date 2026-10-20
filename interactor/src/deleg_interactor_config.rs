use multiversx_sc_snippets::imports::*;
use serde::Deserialize;

/// Chain-specific settings: how to connect, and where the contract lives on that chain.
#[derive(Debug, Deserialize)]
pub struct ChainConfig {
    #[serde(flatten)]
    pub connection: ConnectionConfig,

    /// Address of the delegation contract on this chain.
    ///
    /// Optional, because the chain-simulator harness deploys a fresh contract and only
    /// then knows its address. On a real chain it is always present.
    pub sc_address: Option<Bech32Address>,
}

impl ChainConfig {
    /// The contract address, or a panic explaining how to obtain one.
    pub fn sc_address(&self) -> &Bech32Address {
        self.sc_address
            .as_ref()
            .expect("no `sc_address` set in the config file; deploy a contract first, then set it")
    }
}

/// Gas limits per operation.
///
/// Every field has a default, so the `[gas]` section can be omitted entirely, or specify
/// only the limits that need to differ.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct GasConfig {
    pub deploy: u64,
    pub upgrade: u64,
    pub stake: u64,
    pub unstake: u64,
    pub claim_rewards: u64,
    pub vote: u64,
    pub modify_cap: u64,
    pub continue_global: u64,
    pub fix_users: u64,
}

impl Default for GasConfig {
    fn default() -> Self {
        GasConfig {
            deploy: 300_000_000,
            upgrade: 300_000_000,
            stake: 10_000_000,
            unstake: 30_000_000,
            claim_rewards: 10_000_000,
            vote: 100_000_000,
            modify_cap: 100_000_000,
            continue_global: 400_000_000,
            fix_users: 100_000_000,
        }
    }
}

/// Deployment settings: contract code and constructor arguments.
#[derive(Debug, Deserialize)]
pub struct DeployConfig {
    /// Code of the full contract.
    pub contract_path: ConfigPath,

    /// Auction contract address. The system staking contract, normally.
    pub auction_address: Bech32Address,

    /// Service fee, in 1/10000 (e.g. 1000 = 10%).
    pub service_fee_per_10000: usize,

    /// Minimum share of the stake that the owner must keep, in 1/10000.
    pub owner_min_stake_share_per_10000: usize,

    /// Number of blocks before unbonded funds can be withdrawn.
    pub n_blocks_before_unbond: u64,

    /// Minimum stake, in atomic units (1 EGLD = 10^18). A string, because it can exceed what TOML integers hold.
    pub minimum_stake: String,

    /// Total delegation cap, in atomic units (1 EGLD = 10^18). A string, because it can exceed what TOML integers hold.
    pub total_delegation_cap: String,
}

impl DeployConfig {
    pub fn minimum_stake(&self) -> RustBigUint {
        self.minimum_stake.parse().expect("invalid minimum_stake")
    }

    pub fn total_delegation_cap(&self) -> RustBigUint {
        self.total_delegation_cap
            .parse()
            .expect("invalid total_delegation_cap")
    }
}

/// Upgrade settings.
#[derive(Debug, Deserialize)]
pub struct UpgradeConfig {
    /// Code of the latest contract version.
    pub contract_path: ConfigPath,
}

/// Delegation Interact configuration, for investigating contracts on real chains.
///
/// Everything beyond `chain_config` is optional, so that a config file describing just a
/// chain and a contract address is enough for the read-only commands. Each accessor
/// explains what is missing if a command does need it.
///
/// For work on the chain simulator - the fuzzer and the test actions - see
/// [`crate::test_env::TestConfig`], which is loaded from `test.toml`.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub chain_config: ChainConfig,

    /// Upgrade settings. Needed by `upgrade`.
    pub upgrade: Option<UpgradeConfig>,

    /// Gas limits. Optional, defaults apply per operation.
    #[serde(default)]
    pub gas: GasConfig,

    /// Contract owner, used for upgrading and for admin endpoints.
    pub owner: Option<WalletConfig>,

    /// Main user wallet. Needed by `fix-users`.
    pub user: Option<WalletConfig>,

    /// Sends the `continueGlobalOperation` transactions.
    pub continue_global_sender: Option<WalletConfig>,
}

impl Config {
    pub fn upgrade(&self) -> &UpgradeConfig {
        self.upgrade
            .as_ref()
            .expect("this command needs an `[upgrade]` section in the config file")
    }

    pub fn owner(&self) -> Bech32Address {
        wallet_address(&self.owner, "owner")
    }

    pub fn user(&self) -> Bech32Address {
        wallet_address(&self.user, "user")
    }

    pub fn continue_global_sender(&self) -> Bech32Address {
        wallet_address(&self.continue_global_sender, "continue_global_sender")
    }
}

fn wallet_address(wallet: &Option<WalletConfig>, section: &str) -> Bech32Address {
    wallet
        .as_ref()
        .unwrap_or_else(|| panic!("this command needs a `[{section}]` section in the config file"))
        .address()
}

impl InteractorConfig for Config {
    fn connection(&self) -> &ConnectionConfig {
        &self.chain_config.connection
    }

    fn register_wallets(&self) -> Vec<Wallet> {
        [&self.owner, &self.user, &self.continue_global_sender]
            .into_iter()
            .flatten()
            .map(|wallet_config| wallet_config.wallet().clone())
            .collect()
    }
}
