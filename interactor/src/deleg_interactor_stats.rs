use multiversx_sc_snippets::imports::*;

use crate::{latest_proxy, LegacyDelegationInteractor};

impl LegacyDelegationInteractor {
    pub async fn query_global(&mut self) {
        self.query_total_active_stake().await;
        println!();
        self.query_delegation_cap().await;
        println!();
        self.query_service_fee().await;
        self.query_num_users().await;
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

    pub async fn query_service_fee(&mut self) {
        let service_fee = self
            .interactor
            .query()
            .to(&self.config.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_service_fee()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Service fee: {}", display_percentage(service_fee));
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

        println!("Number of users: {num_users}");
    }

    pub async fn query_settings(&mut self) {
        let auction_contract_address = self
            .interactor
            .query()
            .to(&self.config.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_auction_contract_address()
            .returns(ReturnsResult)
            .run()
            .await;
        let auction_contract_address = Bech32Address::from(auction_contract_address.to_address());

        let service_fee = self
            .interactor
            .query()
            .to(&self.config.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_service_fee()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        let total_delegation_cap = self
            .interactor
            .query()
            .to(&self.config.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_total_delegation_cap()
            .returns(ReturnsResult)
            .run()
            .await;

        let is_bootstrap_mode = self
            .interactor
            .query()
            .to(&self.config.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .is_bootstrap_mode()
            .returns(ReturnsResult)
            .run()
            .await;

        let owner_min_stake_share = self
            .interactor
            .query()
            .to(&self.config.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_owner_min_stake_share()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        let num_blocks_before_unbond = self
            .interactor
            .query()
            .to(&self.config.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_n_blocks_before_unbond()
            .returns(ReturnsResult)
            .run()
            .await;

        let minimum_stake = self
            .interactor
            .query()
            .to(&self.config.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_minimum_stake()
            .returns(ReturnsResult)
            .run()
            .await;

        println!("Auction contract address: {auction_contract_address}");
        println!(
            "Service fee:              {}",
            display_percentage(service_fee)
        );
        println!(
            "Total delegation cap:     {}",
            display_egld_amount(&total_delegation_cap)
        );
        println!("Bootstrap mode:           {is_bootstrap_mode}");
        println!(
            "Owner min stake share:    {}",
            display_percentage(owner_min_stake_share)
        );
        println!("Num blocks before unbond: {num_blocks_before_unbond}");
        println!(
            "Minimum stake:            {}",
            display_egld_amount(&minimum_stake)
        );
    }

    pub async fn query_delegation_cap(&mut self) {
        let delegation_cap = self
            .interactor
            .query()
            .to(&self.config.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_total_delegation_cap()
            .returns(ReturnsResult)
            .run()
            .await;

        println!("Delegation cap: {}", display_egld_amount(&delegation_cap));
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

        let voting_power = self
            .interactor
            .query()
            .to(&self.config.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_voting_power(address)
            .returns(ReturnsResult)
            .run()
            .await;

        println!();
        println!("Voting power:      {}", display_egld_amount(&voting_power));
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

/// Formats a value expressed in hundredths of a percent (10000 = 100%) as e.g. "12.50%".
fn display_percentage<T>(value_per_10000: T) -> String
where
    T: Clone
        + core::ops::Div<u32, Output = T>
        + core::ops::Rem<u32, Output = T>
        + core::fmt::Display,
{
    let whole = value_per_10000.clone() / 100u32;
    let frac = value_per_10000 % 100u32;
    format!("{whole}.{frac:02}%")
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
