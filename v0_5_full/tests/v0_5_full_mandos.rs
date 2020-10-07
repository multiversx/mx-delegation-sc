

use auction_mock::*;
extern crate delegation_v0_5_full;
use delegation_v0_5_full::*;
use elrond_wasm::*;
use elrond_wasm_debug::*;

fn contract_map() -> ContractMap<TxContext> {
    let mut contract_map = ContractMap::new();
    contract_map.register_contract(
        "file:../../../auction-mock/output/auction-mock.wasm",
        Box::new(|context| Box::new(AuctionMockImpl::new(context))));

    contract_map.register_contract(
        "file:../../output/delegation_v0_5_full.wasm",
        Box::new(|context| Box::new(DelegationImpl::new(context))));
    contract_map
}

#[test]
fn activate_nodes() {
    parse_execute_mandos("mandos/activate_nodes.scen.json", &contract_map());
}

#[test]
fn change_service_fee() {
    parse_execute_mandos("mandos/change_service_fee.scen.json", &contract_map());
}

#[test]
fn claim_rewards_1() {
    parse_execute_mandos("mandos/claim_rewards_1.scen.json", &contract_map());
}

#[test]
fn claim_rewards_owner_with_stake() {
    parse_execute_mandos("mandos/claim_rewards_owner_with_stake.scen.json", &contract_map());
}

#[test]
fn claim_rewards_with_changed_service_fee() {
    parse_execute_mandos("mandos/claim_rewards_with_changed_service_fee.scen.json", &contract_map());
}

#[test]
fn claim_rewards_with_modify_delegation_cap() {
    parse_execute_mandos("mandos/claim_rewards_with_modify_delegation_cap.scen.json", &contract_map());
}

#[test]
fn claim_rewards_with_stake_modifications() {
    parse_execute_mandos("mandos/claim_rewards_with_stake_modifications.scen.json", &contract_map());
}

#[test]
fn continue_global_operations() {
    parse_execute_mandos("mandos/continue_global_operations.scen.json", &contract_map());
}

#[test]
fn decrease_cap_in_bootstrap_mode() {
    parse_execute_mandos("mandos/decrease_cap_in_bootstrap_mode.scen.json", &contract_map());
}

#[test]
fn increase_delegation_cap() {
    parse_execute_mandos("mandos/increase_delegation_cap.scen.json", &contract_map());
}

#[test]
fn rewards_for_unstaked_go_to_the_owner() {
    parse_execute_mandos("mandos/rewards_for_unStaked_go_to_the_owner.scen.json", &contract_map());
}

#[test]
fn staking_1() {
    parse_execute_mandos("mandos/staking_1.scen.json", &contract_map());
}

#[test]
fn staking_2() {
    parse_execute_mandos("mandos/staking_2.scen.json", &contract_map());
}

#[test]
fn total_funds_getters() {
    parse_execute_mandos("mandos/total_funds_getters.scen.json", &contract_map());
}

#[test]
fn unbond_from_waiting() {
    parse_execute_mandos("mandos/unbond_from_waiting.scen.json", &contract_map());
}

#[test]
fn unbond() {
    parse_execute_mandos("mandos/unbond.scen.json", &contract_map());
}

#[test]
fn unjail() {
    parse_execute_mandos("mandos/unjail.scen.json", &contract_map());
}

#[test]
fn unstake_1() {
    parse_execute_mandos("mandos/unstake_1.scen.json", &contract_map());
}

#[test]
fn unstake_2() {
    parse_execute_mandos("mandos/unstake_2.scen.json", &contract_map());
}

#[test]
fn unstake_3() {
    parse_execute_mandos("mandos/unstake_3.scen.json", &contract_map());
}

#[test]
fn unstake_4() {
    parse_execute_mandos("mandos/unstake_4.scen.json", &contract_map());
}

#[test]
fn user_fund_getters() {
    parse_execute_mandos("mandos/user_fund_getters.scen.json", &contract_map());
}
