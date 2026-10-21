use multiversx_sc_snippets::imports::*;

use crate::{
    LegacyDelegationInteractor,
    display::{display_egld_amount, display_percentage},
    ops::StakeByType,
};

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
        let stake_by_type = self.ops().total_stake_by_type().await;
        print_stake_by_type(&stake_by_type);
    }

    pub async fn query_service_fee(&mut self) {
        let service_fee = self.ops().service_fee().await;
        println!("Service fee: {}", display_percentage(service_fee));
    }

    pub async fn query_num_users(&mut self) {
        let num_users = self.ops().num_users().await;
        println!("Number of users: {num_users}");
    }

    pub async fn query_settings(&mut self) {
        let auction_contract_address = self.ops().auction_contract_address().await;
        let service_fee = self.ops().service_fee().await;
        let total_delegation_cap = self.ops().total_delegation_cap().await;
        let is_bootstrap_mode = self.ops().is_bootstrap_mode().await;
        let owner_min_stake_share = self.ops().owner_min_stake_share().await;
        let num_blocks_before_unbond = self.ops().n_blocks_before_unbond().await;
        let minimum_stake = self.ops().minimum_stake().await;

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
        let delegation_cap = self.ops().total_delegation_cap().await;
        println!("Delegation cap: {}", display_egld_amount(&delegation_cap));
    }

    pub async fn query_user_stake_by_type(&mut self, address: &Bech32Address) {
        let (withdraw, waiting, active, unstaked, deferred) =
            self.ops().user_stake_by_type(address).await;

        println!("WithdrawOnly:      {}", display_egld_amount(&withdraw));
        println!("Waiting:           {}", display_egld_amount(&waiting));
        println!("Active:            {}", display_egld_amount(&active));
        println!("UnStaked:          {}", display_egld_amount(&unstaked));
        println!("DeferredPayment:   {}", display_egld_amount(&deferred));

        println!();

        let total_unstakeable = self.ops().unstakeable(address).await;
        println!(
            "Total Unstakeable: {}",
            display_egld_amount(&total_unstakeable)
        );

        let total_unbondable = self.ops().unbondable(address).await;
        println!(
            "Total Unbondable:  {}",
            display_egld_amount(&total_unbondable)
        );

        if deferred > RustBigUint::default() {
            println!();

            let num_blocks_before_unbond = self.ops().n_blocks_before_unbond().await;
            println!("Num blocks before unbond: {num_blocks_before_unbond}");

            let deferred_payment_list = self.ops().user_deferred_payment_list(address).await;

            println!("DeferredPayment list:");
            for (amount, reg_block) in deferred_payment_list {
                println!(
                    "Amount:            {}    Registration block: {}    Due block: {}",
                    display_egld_amount(&amount),
                    reg_block,
                    reg_block + num_blocks_before_unbond,
                );
            }
        }

        let voting_power = self.ops().voting_power(address).await;

        println!();
        println!("Voting power:      {}", display_egld_amount(&voting_power));
    }

    pub async fn query_all_user_stake_by_type(&mut self) {
        let all_stake = self.ops().all_user_stake_by_type().await;

        for (address, stake_by_type) in all_stake {
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

/// Prints the contract's five fund buckets, in the order the contract returns them.
pub fn print_stake_by_type(stake_by_type: &StakeByType) {
    println!("WithdrawOnly:    {}", display_egld_amount(&stake_by_type.0));
    println!("Waiting:         {}", display_egld_amount(&stake_by_type.1));
    println!("Active:          {}", display_egld_amount(&stake_by_type.2));
    println!("UnStaked:        {}", display_egld_amount(&stake_by_type.3));
    println!("DeferredPayment: {}", display_egld_amount(&stake_by_type.4));
}
