use elrond_wasm_debug::*;

fn world() -> BlockchainMock {
    let mut blockchain = BlockchainMock::new();
    blockchain.set_current_dir_from_workspace("latest_full");
    blockchain.register_contract_builder(
        "file:../auction-mock/output/auction-mock.wasm",
        auction_mock::ContractBuilder,
    );
    blockchain.register_contract_builder(
        "file:output/delegation_latest_full.wasm",
        delegation_latest_full::ContractBuilder,
    );
    blockchain
}

#[test]
fn activate_nodes_rs() {
    elrond_wasm_debug::mandos_rs("mandos/activate_nodes.scen.json", world());
}

#[test]
fn change_service_fee_rs() {
    elrond_wasm_debug::mandos_rs("mandos/change_service_fee.scen.json", world());
}

#[test]
fn claim_rewards_1_rs() {
    elrond_wasm_debug::mandos_rs("mandos/claim_rewards_1.scen.json", world());
}

#[test]
fn claim_rewards_owner_with_stake_rs() {
    elrond_wasm_debug::mandos_rs("mandos/claim_rewards_owner_with_stake.scen.json", world());
}

#[test]
fn claim_rewards_with_changed_service_fee_rs() {
    elrond_wasm_debug::mandos_rs(
        "mandos/claim_rewards_with_changed_service_fee.scen.json",
        world(),
    );
}

#[test]
fn claim_rewards_with_modify_delegation_cap_rs() {
    elrond_wasm_debug::mandos_rs(
        "mandos/claim_rewards_with_modify_delegation_cap.scen.json",
        world(),
    );
}

#[test]
fn claim_rewards_with_stake_modifications_rs() {
    elrond_wasm_debug::mandos_rs(
        "mandos/claim_rewards_with_stake_modifications.scen.json",
        world(),
    );
}

#[test]
fn continue_global_operations_rs() {
    elrond_wasm_debug::mandos_rs("mandos/continue_global_operations.scen.json", world());
}

#[test]
fn decrease_cap_in_bootstrap_mode_rs() {
    elrond_wasm_debug::mandos_rs("mandos/decrease_cap_in_bootstrap_mode.scen.json", world());
}

#[test]
fn increase_delegation_cap_rs() {
    elrond_wasm_debug::mandos_rs("mandos/increase_delegation_cap.scen.json", world());
}

#[test]
fn rewards_for_unstaked_go_to_the_owner_rs() {
    elrond_wasm_debug::mandos_rs(
        "mandos/rewards_for_unStaked_go_to_the_owner.scen.json",
        world(),
    );
}

#[test]
fn set_num_blocks_before_unbond_rs() {
    elrond_wasm_debug::mandos_rs("mandos/set_num_blocks_before_unbond.scen.json", world());
}

#[test]
fn staking_1_rs() {
    elrond_wasm_debug::mandos_rs("mandos/staking_1.scen.json", world());
}

#[test]
fn staking_2_rs() {
    elrond_wasm_debug::mandos_rs("mandos/staking_2.scen.json", world());
}

#[test]
fn total_funds_getters_rs() {
    elrond_wasm_debug::mandos_rs("mandos/total_funds_getters.scen.json", world());
}

#[test]
fn unbond_rs() {
    elrond_wasm_debug::mandos_rs("mandos/unbond.scen.json", world());
}

#[test]
fn unbond_from_waiting_rs() {
    elrond_wasm_debug::mandos_rs("mandos/unbond_from_waiting.scen.json", world());
}

#[test]
fn unjail_rs() {
    elrond_wasm_debug::mandos_rs("mandos/unjail.scen.json", world());
}

#[test]
fn unstake_1_rs() {
    elrond_wasm_debug::mandos_rs("mandos/unstake_1.scen.json", world());
}

#[test]
fn unstake_2_rs() {
    elrond_wasm_debug::mandos_rs("mandos/unstake_2.scen.json", world());
}

#[test]
fn unstake_3_rs() {
    elrond_wasm_debug::mandos_rs("mandos/unstake_3.scen.json", world());
}

// #[test]
// fn unstake_4_rs() {
//     elrond_wasm_debug::mandos_rs("mandos/unstake_4.scen.json", &contract_map());
// }

#[test]
fn unstake_5_backwards_rs() {
    elrond_wasm_debug::mandos_rs("mandos/unstake_5_backwards.scen.json", world());
}

#[test]
fn user_fund_getters_rs() {
    elrond_wasm_debug::mandos_rs("mandos/user_fund_getters.scen.json", world());
}

#[test]
fn version_rs() {
    elrond_wasm_debug::mandos_rs("mandos/version.scen.json", world());
}
