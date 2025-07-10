mod deleg_interact_admin;
mod deleg_interact_cli;
mod deleg_interact_config;
mod deleg_interact_stats;
mod deleg_interact_user;
mod latest_proxy;

use clap::Parser;
use deleg_interact_cli::{InteractCli, InteractCliCommand};
use deleg_interact_config::Config;

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
        InteractCliCommand::Upgrade => {
            basic_interact.upgrade_contract_to_latest().await;
        }
        InteractCliCommand::FixUsers => {
            basic_interact.fix_users().await;
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
