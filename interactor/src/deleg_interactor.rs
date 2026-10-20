mod deleg_interactor_admin;
mod deleg_interactor_cli;
mod deleg_interactor_config;
mod deleg_interactor_stats;
mod display;
mod latest_proxy;
mod ops;
mod test_env;

use clap::Parser;
use deleg_interactor_cli::{FuzzCliCommand, InteractCli, InteractCliCommand};
use deleg_interactor_config::Config;
use ops::DelegationOps;
use test_env::TestInteractor;

use multiversx_sc_snippets::imports::*;

#[tokio::main]
async fn main() {
    env_logger::init();

    let cli = InteractCli::parse();
    let command = cli.command.expect("interactor command expected");
    let config_file = cli
        .config
        .unwrap_or_else(|| command.default_config_file().to_owned());

    use InteractCliCommand::*;
    match command {
        Version => real(&config_file).await.version().await,
        Global => real(&config_file).await.query_global().await,
        Settings => real(&config_file).await.query_settings().await,
        UserFull => real(&config_file).await.query_all_user_stake_by_type().await,
        NumUsers => real(&config_file).await.query_num_users().await,
        UserStake(args) => {
            let address = Bech32Address::from_bech32_string(args.address);
            real(&config_file)
                .await
                .query_user_stake_by_type(&address)
                .await
        },
        Upgrade => real(&config_file).await.upgrade_contract_to_latest().await,
        ContinueGlobal => real(&config_file).await.continue_global_operation().await,
        FixUsers => real(&config_file).await.fix_users().await,

        Fuzz(fuzz_command) => run_fuzz(fuzz_command, &config_file).await,
    }
}

/// Everything under `fuzz`, running against a chain simulator.
async fn run_fuzz(command: FuzzCliCommand, config_file: &str) {
    let mut interact = TestInteractor::init(config_file).await;

    use FuzzCliCommand::*;
    match command {
        Setup => interact.fuzz_setup().await,
        Deploy => interact.deploy().await,
        Status => interact.status().await,
        Stake(args) => interact.stake(args.amount, args.delegator).await,
        Unstake(args) => interact.unstake(args.amount, args.delegator).await,
        Unbond(args) => interact.unbond(args.delegator).await,
        Claim(args) => interact.claim_rewards(args.delegator).await,
        Vote(args) => interact.delegate_vote(args.delegator).await,
        AddNodes(args) => interact.add_nodes(args.count).await,
        RemoveNodes(args) => interact.remove_nodes(args.count).await,
        SetServiceFee(args) => interact.set_service_fee(args.service_fee_per_10000).await,
        ModifyCap(args) => interact.modify_delegation_cap(args.amount).await,
        ContinueGlobal => interact.continue_global_operation().await,
        DustCleanup(args) => interact.dust_cleanup(args.amount).await,
        AddRewards(args) => interact.add_rewards(args.amount).await,
        GenerateBlocks(args) => interact.generate_blocks(args.num_blocks).await,
        CheckInvariants => interact.check_invariants().await,
    }
}

async fn real(config_file: &str) -> LegacyDelegationInteractor {
    LegacyDelegationInteractor::init(config_file).await
}

/// Interactor for investigating delegation contracts deployed on real chains.
pub struct LegacyDelegationInteractor {
    interactor: Interactor,
    config: Config,
}

impl LegacyDelegationInteractor {
    async fn init(config_file: &str) -> Self {
        let mut interactor = Interactor::empty().with_current_dir(env!("CARGO_MANIFEST_DIR"));
        let config: Config = interactor.load_config_from_file(config_file).await;

        LegacyDelegationInteractor { interactor, config }
    }

    fn ops(&mut self) -> DelegationOps<'_> {
        DelegationOps::new(
            &mut self.interactor,
            self.config.chain_config.sc_address(),
            &self.config.gas,
        )
    }
}
