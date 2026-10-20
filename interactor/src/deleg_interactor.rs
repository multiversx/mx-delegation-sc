mod deleg_interactor_admin;
mod deleg_interactor_cli;
mod deleg_interactor_config;
mod deleg_interactor_stats;
mod display;
mod latest_proxy;
mod ops;
mod test_env;

use clap::Parser;
use deleg_interactor_cli::{InteractCli, InteractCliCommand};
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
        // --- real chain investigation ---
        Version => real(&config_file).await.version().await,
        Global => real(&config_file).await.query_global().await,
        Settings => real(&config_file).await.query_settings().await,
        UserFull => {
            real(&config_file)
                .await
                .query_all_user_stake_by_type()
                .await
        }
        NumUsers => real(&config_file).await.query_num_users().await,
        UserStake(args) => {
            let address = Bech32Address::from_bech32_string(args.address);
            real(&config_file)
                .await
                .query_user_stake_by_type(&address)
                .await
        }
        Upgrade => real(&config_file).await.upgrade_contract_to_latest().await,
        ContinueGlobal => real(&config_file).await.continue_global_operation().await,
        FixUsers => real(&config_file).await.fix_users().await,

        // --- chain simulator testing ---
        Deploy => test(&config_file).await.deploy().await,
        Stake(args) => test(&config_file).await.stake(args.amount).await,
        Unstake(args) => test(&config_file).await.unstake(args.amount).await,
        Claim => test(&config_file).await.claim_rewards().await,
        Vote => test(&config_file).await.delegate_vote().await,
        ModifyCap(args) => {
            test(&config_file)
                .await
                .modify_delegation_cap(args.amount)
                .await
        }
        AddRewards(args) => test(&config_file).await.add_rewards(args.amount).await,
        GenerateBlocks(args) => {
            test(&config_file)
                .await
                .generate_blocks(args.num_blocks)
                .await
        }
    }
}

async fn real(config_file: &str) -> LegacyDelegationInteractor {
    LegacyDelegationInteractor::init(config_file).await
}

async fn test(config_file: &str) -> TestInteractor {
    TestInteractor::init(config_file).await
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
