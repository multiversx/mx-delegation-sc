use clap::{Args, Parser, Subcommand};

/// Adder Interact CLI
#[derive(Default, PartialEq, Eq, Debug, Parser)]
#[command(version, about)]
#[command(propagate_version = true)]
pub struct InteractCli {
    #[command(subcommand)]
    pub command: Option<InteractCliCommand>,
}

/// Adder Interact CLI Commands
#[derive(Clone, PartialEq, Eq, Debug, Subcommand)]
pub enum InteractCliCommand {
    #[command(name = "version", about = "Query global state")]
    Version,

    #[command(name = "global", about = "Query global state")]
    Global,

    #[command(name = "user-full", about = "All user stake by type")]
    UserFull,

    #[command(name = "num-users", about = "User address list")]
    NumUsers,

    #[command(name = "user-stake", about = "User stake by type")]
    UserStake(UserStakeArgs),

    #[command(about = "Stake for a user")]
    Stake,

    #[command(about = "Unstake for a user")]
    Unstake,

    #[command(about = "Claim rewards")]
    Claim,

    #[command(about = "Vote")]
    Vote,

    #[command(about = "Upgrade contract to latest")]
    Upgrade,

    #[command(about = "Modify delegation cap")]
    ModifyCap,

    #[command(about = "Fix missing info from the old contract")]
    FixUsers,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct UserStakeArgs {
    /// The user bech32 address
    #[arg()]
    pub address: String,
}
