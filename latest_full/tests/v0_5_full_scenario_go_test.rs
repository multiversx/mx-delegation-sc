use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    ScenarioWorld::vm_go()
}

#[test]
fn activate_nodes_go() {
    world().run("scenarios/activate_nodes.scen.json");
}

#[test]
fn change_service_fee_go() {
    world().run("scenarios/change_service_fee.scen.json");
}

#[test]
fn claim_rewards_1_go() {
    world().run("scenarios/claim_rewards_1.scen.json");
}

#[test]
fn claim_rewards_owner_with_stake_go() {
    world().run("scenarios/claim_rewards_owner_with_stake.scen.json");
}

#[test]
fn claim_rewards_with_changed_service_fee_go() {
    world().run("scenarios/claim_rewards_with_changed_service_fee.scen.json");
}

#[test]
fn claim_rewards_with_modify_delegation_cap_go() {
    world().run("scenarios/claim_rewards_with_modify_delegation_cap.scen.json");
}

#[test]
fn claim_rewards_with_stake_modifications_go() {
    world().run("scenarios/claim_rewards_with_stake_modifications.scen.json");
}

#[test]
fn continue_global_operations_go() {
    world().run("scenarios/continue_global_operations.scen.json");
}

#[test]
fn decrease_cap_in_bootstrap_mode_go() {
    world().run("scenarios/decrease_cap_in_bootstrap_mode.scen.json");
}

#[test]
fn increase_delegation_cap_go() {
    world().run("scenarios/increase_delegation_cap.scen.json");
}

#[test]
fn rewards_for_un_staked_go_to_the_owner_go() {
    world().run("scenarios/rewards_for_unStaked_go_to_the_owner.scen.json");
}

#[test]
fn set_num_blocks_before_unbond_go() {
    world().run("scenarios/set_num_blocks_before_unbond.scen.json");
}

#[test]
fn staking_1_go() {
    world().run("scenarios/staking_1.scen.json");
}

#[test]
fn staking_2_go() {
    world().run("scenarios/staking_2.scen.json");
}

#[test]
fn total_funds_getters_go() {
    world().run("scenarios/total_funds_getters.scen.json");
}

#[test]
fn unbond_go() {
    world().run("scenarios/unbond.scen.json");
}

#[test]
fn unbond_from_waiting_go() {
    world().run("scenarios/unbond_from_waiting.scen.json");
}

#[test]
fn unjail_go() {
    world().run("scenarios/unjail.scen.json");
}

#[test]
fn unstake_1_go() {
    world().run("scenarios/unstake_1.scen.json");
}

#[test]
fn unstake_2_go() {
    world().run("scenarios/unstake_2.scen.json");
}

#[test]
fn unstake_3_go() {
    world().run("scenarios/unstake_3.scen.json");
}

#[test]
fn unstake_4_go() {
    world().run("scenarios/unstake_4.scen.json");
}

#[test]
fn unstake_5_backwards_go() {
    world().run("scenarios/unstake_5_backwards.scen.json");
}

#[test]
fn unstake_tokens_go() {
    world().run("scenarios/unstake_tokens.scen.json");
}

#[test]
fn user_fund_getters_go() {
    world().run("scenarios/user_fund_getters.scen.json");
}

#[test]
fn version_go() {
    world().run("scenarios/version.scen.json");
}
