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
    #[command(name = "global", about = "Query global state")]
    Global,

    #[command(name = "user-full", about = "All user stake by type")]
    UserFull,

    #[command(name = "num-users", about = "User address list")]
    NumUsers,

    #[command(name = "user-stake", about = "User stake by type")]
    UserStake(UserStakeArgs),
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct UserStakeArgs {
    /// The user bech32 address
    #[arg()]
    pub address: String,
}
