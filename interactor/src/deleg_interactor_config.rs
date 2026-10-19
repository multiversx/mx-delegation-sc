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

/// General settings, not chain-specific.
#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    /// Code of the latest contract version, used for upgrades.
    pub contract_path: ConfigPath,
}

/// Delegation Interact configuration
#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
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
