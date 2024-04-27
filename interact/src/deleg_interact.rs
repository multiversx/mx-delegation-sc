mod deleg_interact_cli;
mod deleg_interact_config;
mod latest_proxy;

use clap::Parser;
use deleg_interact_cli::{InteractCli, InteractCliCommand};
use deleg_interact_config::Config;

use multiversx_sc_snippets::imports::*;

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut basic_interact = AdderInteract::init().await;

    let cli = InteractCli::parse();
    match &cli.command.expect("interactor command expected") {
        InteractCliCommand::Global => {
            basic_interact.query_global().await;
        }
    }
}

#[allow(unused)]
struct AdderInteract {
    interactor: Interactor,
    config: Config,
}

impl AdderInteract {
    async fn init() -> Self {
        let config = Config::load_config();
        let interactor = Interactor::new(config.gateway()).await;

        Self { interactor, config }
    }

    async fn query_global(&mut self) {
        self.query_total_active_stake().await;
    }

    async fn query_total_active_stake(&mut self) {
        let result = self
            .interactor
            .query()
            .to(&self.config.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_total_stake_by_type_endpoint()
            .returns(ReturnsResult)
            .prepare_async()
            .run()
            .await;

        let tuple = result.into_tuple();

        println!("WithdrawOnly:    {}", display_egld_amount(&tuple.0));
        println!("Waiting:         {}", display_egld_amount(&tuple.1));
        println!("Active:          {}", display_egld_amount(&tuple.2));
        println!("UnStaked:        {}", display_egld_amount(&tuple.3));
        println!("DeferredPayment: {}", display_egld_amount(&tuple.4));
    }
}

fn display_egld_amount(managed_bu: &BigUint<StaticApi>) -> String {
    if managed_bu == &0u32 {
        return "       0".to_string();
    }

    let s = managed_bu.to_alloc().to_string();
    let s = format!("{s:0>19}");
    let s = format!("{s:>26}");
    let len_before_dot = s.len() - 18;
    format!("{}.{}", &s[..len_before_dot], &s[len_before_dot..])
}
