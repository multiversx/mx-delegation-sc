use elrond_wasm::*;
use elrond_wasm_debug::*;

fn contract_map() -> ContractMap<TxContext> {
    let mut contract_map = ContractMap::new();
    contract_map.register_contract(
        "file:../../../auction-mock/output/auction-mock.wasm",
        Box::new(|context| Box::new(auction_mock::contract_obj(context))),
    );

    contract_map.register_contract(
        "file:../../output/delegation_latest_full.wasm",
        Box::new(|context| Box::new(delegation_latest_full::contract_obj(context))),
    );
    contract_map.register_contract(
        "file:../output/delegation_latest_full.wasm",
        Box::new(|context| Box::new(delegation_latest_full::contract_obj(context))),
    );
    contract_map
}

#[test]
fn set_num_blocks_before_unbond() {
    elrond_wasm_debug::mandos_rs(
        "mandos/set_num_blocks_before_unbond.scen.json",
        &contract_map(),
    );
}

#[test]
fn activate_nodes() {
    elrond_wasm_debug::mandos_rs("mandos/activate_nodes.scen.json", &contract_map());
}

#[test]
fn change_service_fee() {
    elrond_wasm_debug::mandos_rs("mandos/change_service_fee.scen.json", &contract_map());
}

#[test]
fn claim_rewards_1() {
    elrond_wasm_debug::mandos_rs("mandos/claim_rewards_1.scen.json", &contract_map());
}

#[test]
fn claim_rewards_owner_with_stake() {
    elrond_wasm_debug::mandos_rs(
        "mandos/claim_rewards_owner_with_stake.scen.json",
        &contract_map(),
    );
}

#[test]
fn claim_rewards_with_changed_service_fee() {
    elrond_wasm_debug::mandos_rs(
        "mandos/claim_rewards_with_changed_service_fee.scen.json",
        &contract_map(),
    );
}

#[test]
fn claim_rewards_with_modify_delegation_cap() {
    elrond_wasm_debug::mandos_rs(
        "mandos/claim_rewards_with_modify_delegation_cap.scen.json",
        &contract_map(),
    );
}

#[test]
fn claim_rewards_with_stake_modifications() {
    elrond_wasm_debug::mandos_rs(
        "mandos/claim_rewards_with_stake_modifications.scen.json",
        &contract_map(),
    );
}

#[test]
fn continue_global_operations() {
    elrond_wasm_debug::mandos_rs(
        "mandos/continue_global_operations.scen.json",
        &contract_map(),
    );
}

#[test]
fn decrease_cap_in_bootstrap_mode() {
    elrond_wasm_debug::mandos_rs(
        "mandos/decrease_cap_in_bootstrap_mode.scen.json",
        &contract_map(),
    );
}

#[test]
fn increase_delegation_cap() {
    elrond_wasm_debug::mandos_rs("mandos/increase_delegation_cap.scen.json", &contract_map());
}

#[test]
fn rewards_for_unstaked_go_to_the_owner() {
    elrond_wasm_debug::mandos_rs(
        "mandos/rewards_for_unStaked_go_to_the_owner.scen.json",
        &contract_map(),
    );
}

#[test]
fn staking_1() {
    elrond_wasm_debug::mandos_rs("mandos/staking_1.scen.json", &contract_map());
}

#[test]
fn staking_2() {
    elrond_wasm_debug::mandos_rs("mandos/staking_2.scen.json", &contract_map());
}

#[test]
fn total_funds_getters() {
    elrond_wasm_debug::mandos_rs("mandos/total_funds_getters.scen.json", &contract_map());
}

#[test]
fn unbond_from_waiting() {
    elrond_wasm_debug::mandos_rs("mandos/unbond_from_waiting.scen.json", &contract_map());
}

#[test]
fn unbond() {
    elrond_wasm_debug::mandos_rs("mandos/unbond.scen.json", &contract_map());
}

#[test]
fn unjail() {
    elrond_wasm_debug::mandos_rs("mandos/unjail.scen.json", &contract_map());
}

#[test]
fn unstake_1() {
    elrond_wasm_debug::mandos_rs("mandos/unstake_1.scen.json", &contract_map());
}

#[test]
fn unstake_2() {
    elrond_wasm_debug::mandos_rs("mandos/unstake_2.scen.json", &contract_map());
}

#[test]
fn unstake_3() {
    elrond_wasm_debug::mandos_rs("mandos/unstake_3.scen.json", &contract_map());
}

// TODO: uncomment after upgrading to 0.14. scQuery still missing here
// #[test]
// fn unstake_4() {
//     elrond_wasm_debug::mandos_rs("mandos/unstake_4.scen.json", &contract_map());
// }

#[test]
fn unstake_5_backwards() {
    elrond_wasm_debug::mandos_rs("mandos/unstake_5_backwards.scen.json", &contract_map());
}

#[test]
fn user_fund_getters() {
    elrond_wasm_debug::mandos_rs("mandos/user_fund_getters.scen.json", &contract_map());
}
