use multiversx_sc_snippets::imports::*;
use serde::Deserialize;

/// General settings: which contract, and where its code is.
#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    /// Address of the delegation contract.
    pub sc_address: Bech32Address,

    /// Code of the latest contract version, used for upgrades.
    pub contract_path: ConfigPath,
}

/// Delegation Interact configuration
#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub connection: ConnectionConfig,

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
        &self.connection
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
