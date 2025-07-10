use multiversx_sc_snippets::imports::*;

use crate::{latest_proxy, LegacyDelegationInteractor};

impl LegacyDelegationInteractor {
    pub async fn query_global(&mut self) {
        self.query_total_active_stake().await;
    }

    pub async fn query_total_active_stake(&mut self) {
        let result = self
            .interactor
            .query()
            .to(&self.config.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_total_stake_by_type_endpoint()
            .returns(ReturnsResult)
            .run()
            .await;

        let tuple = result.into_tuple();

        println!("WithdrawOnly:    {}", display_egld_amount(&tuple.0));
        println!("Waiting:         {}", display_egld_amount(&tuple.1));
        println!("Active:          {}", display_egld_amount(&tuple.2));
        println!("UnStaked:        {}", display_egld_amount(&tuple.3));
        println!("DeferredPayment: {}", display_egld_amount(&tuple.4));
    }

    pub async fn query_num_users(&mut self) {
        let num_users = self
            .interactor
            .query()
            .to(&self.config.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_num_users()
            .returns(ReturnsResult)
            .run()
            .await;

        println!("{num_users}");
    }

    pub async fn query_user_stake_by_type(&mut self, address: &Bech32Address) {
        let result = self
            .interactor
            .query()
            .to(&self.config.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_user_stake_by_type_endpoint(address)
            .returns(ReturnsResult)
            .run()
            .await;

        let (withdraw, waiting, active, unstaked, deferred) = result.into_tuple();

        println!("WithdrawOnly:      {}", display_egld_amount(&withdraw));
        println!("Waiting:           {}", display_egld_amount(&waiting));
        println!("Active:            {}", display_egld_amount(&active));
        println!("UnStaked:          {}", display_egld_amount(&unstaked));
        println!("DeferredPayment:   {}", display_egld_amount(&deferred));

        println!();

        let total_unstakeable = self
            .interactor
            .query()
            .to(&self.config.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_unstakeable(address)
            .returns(ReturnsResult)
            .run()
            .await;

        println!(
            "Total Unstakeable: {}",
            display_egld_amount(&total_unstakeable)
        );

        let total_unbondable = self
            .interactor
            .query()
            .to(&self.config.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_unbondable(address)
            .returns(ReturnsResult)
            .run()
            .await;

        println!(
            "Total Unbondable:  {}",
            display_egld_amount(&total_unbondable)
        );

        if deferred > 0 {
            println!();

            let num_blocks_before_unbond = self
                .interactor
                .query()
                .to(&self.config.sc_address)
                .typed(latest_proxy::DelegationFullProxy)
                .get_n_blocks_before_unbond()
                .returns(ReturnsResult)
                .run()
                .await;

            println!("Num blocks before unbond: {num_blocks_before_unbond}");

            let result = self
                .interactor
                .query()
                .to(&self.config.sc_address)
                .typed(latest_proxy::DelegationFullProxy)
                .get_user_deferred_payment_list(address)
                .returns(ReturnsResult)
                .run()
                .await;

            println!("DeferredPayment list:");
            for pair in result {
                let (amount, reg_block) = pair.into_tuple();
                println!(
                    "Amount:            {}    Registration block: {}    Due block: {}",
                    display_egld_amount(&amount),
                    reg_block,
                    reg_block + num_blocks_before_unbond,
                );
            }
        }
    }

    pub async fn query_all_user_stake_by_type(&mut self) {
        let result = self
            .interactor
            .query()
            .to(&self.config.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_all_user_stake_by_type()
            .returns(ReturnsResult)
            .run()
            .await;

        for user_stake_tuple in result {
            let tuple = user_stake_tuple.into_tuple();
            let address = Bech32Address::from(tuple.0.to_address());
            let stake_by_type = tuple.1.into_tuple();

            print!("{address}: ");
            print!(" {}", display_egld_amount(&stake_by_type.0));
            print!(" {}", display_egld_amount(&stake_by_type.1));
            print!(" {}", display_egld_amount(&stake_by_type.2));
            print!(" {}", display_egld_amount(&stake_by_type.3));
            print!(" {}", display_egld_amount(&stake_by_type.4));
            println!();
        }
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
