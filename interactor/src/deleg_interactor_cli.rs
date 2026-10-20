use clap::{Args, Parser, Subcommand};

/// Which config file a command reads when `--config` is not given.
pub const REAL_CHAIN_CONFIG: &str = "config.toml";
pub const TEST_CONFIG: &str = "test.toml";

/// Delegation Interact CLI
#[derive(Default, PartialEq, Eq, Debug, Parser)]
#[command(version, about)]
#[command(propagate_version = true)]
pub struct InteractCli {
    /// Config file to load, relative to the interactor directory.
    ///
    /// Defaults per command: `config.toml` for the real-chain investigation commands,
    /// `test.toml` for the chain-simulator ones. Pointing a command at the other file
    /// works as long as that file carries the sections the command needs.
    #[arg(long, global = true)]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Option<InteractCliCommand>,
}

/// Delegation Interact CLI Commands
#[derive(Clone, PartialEq, Eq, Debug, Subcommand)]
pub enum InteractCliCommand {
    // --- real chain investigation, `config.toml` ---
    #[command(name = "version", about = "Query the contract version")]
    Version,

    #[command(name = "global", about = "Query global state")]
    Global,

    #[command(name = "settings", about = "Query all contract settings")]
    Settings,

    #[command(name = "user-full", about = "All user stake by type")]
    UserFull,

    #[command(name = "num-users", about = "Number of users")]
    NumUsers,

    #[command(name = "user-stake", about = "User stake by type")]
    UserStake(UserStakeArgs),

    #[command(about = "Upgrade contract to latest")]
    Upgrade,

    #[command(
        name = "continue-global",
        about = "Continue the global operation until it completes"
    )]
    ContinueGlobal,

    #[command(about = "Fix missing info from the old contract")]
    FixUsers,

    // --- chain simulator testing, `test.toml` ---
    #[command(about = "Deploy a new delegation contract")]
    Deploy,

    #[command(about = "Stake for a user")]
    Stake(AmountArgs),

    #[command(about = "Unstake for a user")]
    Unstake(AmountArgs),

    #[command(about = "Claim rewards")]
    Claim,

    #[command(about = "Vote")]
    Vote,

    #[command(name = "modify-cap", about = "Modify the total delegation cap")]
    ModifyCap(AmountArgs),

    #[command(
        name = "add-rewards",
        about = "Simulate a validator reward arriving at the contract (chain simulator only)"
    )]
    AddRewards(AmountArgs),

    #[command(
        name = "generate-blocks",
        about = "Advance the chain simulator by a number of blocks"
    )]
    GenerateBlocks(BlocksArgs),
}

impl InteractCliCommand {
    /// The config file this command reads when `--config` is not given.
    pub fn default_config_file(&self) -> &'static str {
        use InteractCliCommand::*;
        match self {
            Version | Global | Settings | UserFull | NumUsers | UserStake(_) | Upgrade
            | ContinueGlobal | FixUsers => REAL_CHAIN_CONFIG,

            Deploy | Stake(_) | Unstake(_) | Claim | Vote | ModifyCap(_) | AddRewards(_)
            | GenerateBlocks(_) => TEST_CONFIG,
        }
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct UserStakeArgs {
    /// The user bech32 address
    #[arg()]
    pub address: String,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct AmountArgs {
    /// Amount in atomic units (1 EGLD = 10^18). Defaults to the value in the config file.
    #[arg()]
    pub amount: Option<String>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct BlocksArgs {
    /// Number of blocks to generate.
    #[arg()]
    pub num_blocks: u64,
}
