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
    /// `test.toml` for everything under `fuzz`. Pointing a command at the other file
    /// works as long as that file carries the sections the command needs.
    #[arg(long, global = true)]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Option<InteractCliCommand>,
}

/// Top-level commands: investigating a contract on a real chain, plus the `fuzz` group
/// for everything that runs against a chain simulator.
#[derive(Clone, PartialEq, Eq, Debug, Subcommand)]
pub enum InteractCliCommand {
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

    #[command(
        subcommand,
        about = "Fuzzing and testing against a chain simulator (reads test.toml)"
    )]
    Fuzz(FuzzCliCommand),
}

impl InteractCliCommand {
    /// The config file this command reads when `--config` is not given.
    pub fn default_config_file(&self) -> &'static str {
        match self {
            InteractCliCommand::Fuzz(_) => TEST_CONFIG,
            _ => REAL_CHAIN_CONFIG,
        }
    }
}

/// Everything that runs against a chain simulator: the fuzzer, and the individual
/// operations it is built from, each runnable by hand.
#[derive(Clone, PartialEq, Eq, Debug, Subcommand)]
pub enum FuzzCliCommand {
    #[command(about = "Run the fuzzer: deploy, then a random sequence of operations")]
    Run(FuzzRunArgs),

    #[command(about = "Deploy a contract and prepare the delegators and sinks")]
    Setup,

    #[command(about = "Deploy a new delegation contract, nothing else")]
    Deploy,

    #[command(about = "Contract totals, settings and sink balances")]
    Status,

    #[command(about = "Stake for a delegator")]
    Stake(UserActionArgs),

    #[command(about = "Unstake for a delegator")]
    Unstake(UserActionArgs),

    #[command(about = "Unbond everything available, into the withdraw sink")]
    Unbond(DelegatorArgs),

    #[command(about = "Claim rewards, into the rewards sink")]
    Claim(DelegatorArgs),

    #[command(about = "Vote on a governance proposal")]
    Vote(DelegatorArgs),

    #[command(name = "add-nodes", about = "Register nodes with the contract")]
    AddNodes(CountArgs),

    #[command(name = "remove-nodes", about = "Remove the most recently added nodes")]
    RemoveNodes(CountArgs),

    #[command(name = "set-service-fee", about = "Set the service fee")]
    SetServiceFee(ServiceFeeArgs),

    #[command(name = "modify-cap", about = "Modify the total delegation cap")]
    ModifyCap(AmountArgs),

    #[command(
        name = "continue-global",
        about = "Continue the global operation until it completes"
    )]
    ContinueGlobal,

    #[command(name = "dust-cleanup", about = "Sweep fund entries below a limit")]
    DustCleanup(AmountArgs),

    #[command(
        name = "add-rewards",
        about = "Simulate a validator reward arriving at the contract"
    )]
    AddRewards(AmountArgs),

    #[command(name = "generate-blocks", about = "Advance the simulator by N blocks")]
    GenerateBlocks(BlocksArgs),

    #[command(
        name = "check-invariants",
        about = "Run the contract's own consistency checks"
    )]
    CheckInvariants,
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
pub struct UserActionArgs {
    /// Amount in atomic units (1 EGLD = 10^18). Defaults to the value in the config file.
    #[arg()]
    pub amount: Option<String>,

    /// Act as this generated delegator (0 is the owner) instead of the [user] wallet.
    #[arg(long)]
    pub delegator: Option<usize>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct DelegatorArgs {
    /// Act as this generated delegator (0 is the owner) instead of the [user] wallet.
    #[arg(long)]
    pub delegator: Option<usize>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct FuzzRunArgs {
    /// Random seed. Overrides the config; zero or absent means derive one from the clock.
    #[arg(long)]
    pub seed: Option<u64>,

    /// How many events to generate. Overrides the config.
    #[arg(long)]
    pub iterations: Option<usize>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct CountArgs {
    /// How many nodes.
    #[arg()]
    pub count: usize,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct ServiceFeeArgs {
    /// Service fee in 1/10000 (e.g. 1000 = 10%).
    #[arg()]
    pub service_fee_per_10000: usize,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct BlocksArgs {
    /// Number of blocks to generate.
    #[arg()]
    pub num_blocks: u64,
}
