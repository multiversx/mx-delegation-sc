use multiversx_sc_snippets::imports::*;
use serde::Deserialize;

/// Chain-specific settings: how to connect, and where the contract lives on that chain.
#[derive(Debug, Deserialize)]
pub struct ChainConfig {
    #[serde(flatten)]
    pub connection: ConnectionConfig,

    /// Address of the delegation contract on this chain.
    pub sc_address: Bech32Address,
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

/// Upgrade settings.
#[derive(Debug, Deserialize)]
pub struct UpgradeConfig {
    /// Code of the latest contract version.
    pub contract_path: ConfigPath,
}

/// Delegation Interact configuration
#[derive(Debug, Deserialize)]
pub struct Config {
    pub deploy: DeployConfig,
    pub upgrade: UpgradeConfig,
    pub chain_config: ChainConfig,

    /// Contract owner, used for upgrading and for admin endpoints.
    pub owner: WalletConfig,

    /// Main user wallet, used for staking, unstaking, voting and others.
    pub user: WalletConfig,

    /// Secondary user wallet, used for claiming rewards.
    pub user2: WalletConfig,

    /// Sends the `continueGlobalOperation` transactions.
    pub continue_global_sender: WalletConfig,
}

impl InteractorConfig for Config {
    fn connection(&self) -> &ConnectionConfig {
        &self.chain_config.connection
    }

    fn register_wallets(&self) -> Vec<Wallet> {
        vec![
            self.owner.wallet().clone(),
            self.user.wallet().clone(),
            self.user2.wallet().clone(),
            self.continue_global_sender.wallet().clone(),
        ]
    }
}
