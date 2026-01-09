use multiversx_sc_scenario::imports::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(
        "file:../auction-mock/output/auction-mock.wasm",
        auction_mock::ContractBuilder,
    );

    blockchain.register_partial_contract::<delegation_latest::AbiProvider, _>(
        "file:output/delegation_latest_full.wasm",
        delegation_latest::ContractBuilder,
        "delegation_latest_full",
    );
    blockchain.register_partial_contract::<delegation_latest::AbiProvider, _>(
        "file:output/delegation_latest_update.wasm",
        delegation_latest::ContractBuilder,
        "delegation_latest_update",
    );

    blockchain
}

#[test]
fn activate_nodes_rs() {
    world().run("scenarios/activate_nodes.scen.json");
}

#[test]
fn change_service_fee_rs() {
    world().run("scenarios/change_service_fee.scen.json");
}

#[test]
fn claim_rewards_1_rs() {
    world().run("scenarios/claim_rewards_1.scen.json");
}

#[test]
fn claim_rewards_owner_with_stake_rs() {
    world().run("scenarios/claim_rewards_owner_with_stake.scen.json");
}

#[test]
fn claim_rewards_with_changed_service_fee_rs() {
    world().run("scenarios/claim_rewards_with_changed_service_fee.scen.json");
}

#[test]
fn claim_rewards_with_modify_delegation_cap_rs() {
    world().run("scenarios/claim_rewards_with_modify_delegation_cap.scen.json");
}

#[test]
fn claim_rewards_with_stake_modifications_rs() {
    world().run("scenarios/claim_rewards_with_stake_modifications.scen.json");
}

#[test]
fn continue_global_operations_rs() {
    world().run("scenarios/continue_global_operations.scen.json");
}

#[test]
fn decrease_cap_in_bootstrap_mode_rs() {
    world().run("scenarios/decrease_cap_in_bootstrap_mode.scen.json");
}

#[test]
fn genesis_addr_fix_rs() {
    world().run("scenarios/genesis_addr_fix.scen.json");
}

#[test]
fn increase_delegation_cap_rs() {
    world().run("scenarios/increase_delegation_cap.scen.json");
}

#[test]
fn rewards_for_un_staked_go_to_the_owner_rs() {
    world().run("scenarios/rewards_for_unStaked_go_to_the_owner.scen.json");
}

#[test]
fn set_num_blocks_before_unbond_rs() {
    world().run("scenarios/set_num_blocks_before_unbond.scen.json");
}

#[test]
fn staking_1_rs() {
    world().run("scenarios/staking_1.scen.json");
}

#[test]
fn staking_2_rs() {
    world().run("scenarios/staking_2.scen.json");
}

#[test]
fn total_funds_getters_rs() {
    world().run("scenarios/total_funds_getters.scen.json");
}

#[test]
fn unbond_rs() {
    world().run("scenarios/unbond.scen.json");
}

#[test]
fn unbond_from_waiting_rs() {
    world().run("scenarios/unbond_from_waiting.scen.json");
}

#[test]
fn unjail_rs() {
    world().run("scenarios/unjail.scen.json");
}

#[test]
fn unstake_1_rs() {
    world().run("scenarios/unstake_1.scen.json");
}

#[test]
fn unstake_2_rs() {
    world().run("scenarios/unstake_2.scen.json");
}

#[test]
fn unstake_3_rs() {
    world().run("scenarios/unstake_3.scen.json");
}

#[test]
#[ignore = "gas"]
fn unstake_4_rs() {
    world().run("scenarios/unstake_4.scen.json");
}

#[test]
fn unstake_5_backwards_rs() {
    world().run("scenarios/unstake_5_backwards.scen.json");
}

#[test]
fn unstake_tokens_rs() {
    world().run("scenarios/unstake_tokens.scen.json");
}

#[test]
fn user_fund_getters_rs() {
    world().run("scenarios/user_fund_getters.scen.json");
}

#[test]
fn version_full_rs() {
    world().run("scenarios/version_full.scen.json");
}

#[test]
fn version_update_rs() {
    world().run("scenarios/version_update.scen.json");
}
