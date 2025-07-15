mod deleg_interactor_admin;
mod deleg_interactor_cli;
mod deleg_interactor_config;
mod deleg_interactor_stats;
mod deleg_interactor_user;
mod latest_proxy;

use clap::Parser;
use deleg_interactor_cli::{InteractCli, InteractCliCommand};
use deleg_interactor_config::Config;

use multiversx_sc_snippets::imports::*;

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut basic_interact = LegacyDelegationInteractor::init().await;

    let cli = InteractCli::parse();
    match &cli.command.expect("interactor command expected") {
        InteractCliCommand::Version => {
            basic_interact.version().await;
        }
        InteractCliCommand::Global => {
            basic_interact.query_global().await;
        }
        InteractCliCommand::UserFull => {
            basic_interact.query_all_user_stake_by_type().await;
        }
        InteractCliCommand::NumUsers => basic_interact.query_num_users().await,
        InteractCliCommand::UserStake(args) => {
            let address = Bech32Address::from_bech32_string(args.address.clone());
            basic_interact.query_user_stake_by_type(&address).await;
        }
        InteractCliCommand::Stake => {
            basic_interact.stake_endpoint().await;
        }
        InteractCliCommand::Unstake => {
            basic_interact.unstake_endpoint().await;
        }
        InteractCliCommand::Claim => {
            basic_interact.claim_rewards().await;
        }
        InteractCliCommand::Vote => {
            basic_interact.delegate_vote().await;
        }
        InteractCliCommand::Upgrade => {
            basic_interact.upgrade_contract_to_latest().await;
        }
        InteractCliCommand::FixUsers => {
            basic_interact.fix_users().await;
        }
        InteractCliCommand::ModifyCap => {
            basic_interact.modify_delegation_cap().await;
        }
    }
}

#[allow(unused)]
struct LegacyDelegationInteractor {
    interactor: Interactor,
    config: Config,
}

impl LegacyDelegationInteractor {
    async fn init() -> Self {
        let config = Config::load_config();
        let interactor = Interactor::new(config.gateway()).await;

        Self { interactor, config }
    }
}
