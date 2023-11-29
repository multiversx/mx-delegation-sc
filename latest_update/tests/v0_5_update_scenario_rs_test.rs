use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.set_current_dir_from_workspace("latest_update");
    blockchain.register_contract(
        "file:../auction-mock/output/auction-mock.wasm",
        auction_mock::ContractBuilder,
    );
    blockchain.register_contract(
        "file:output/delegation_latest_update.wasm",
        delegation_latest_update::ContractBuilder,
    );
    blockchain
}

#[test]
fn genesis_addr_fix_rs() {
    world().run("scenarios/genesis_addr_fix.scen.json");
}

#[test]
fn version_rs() {
    world().run("scenarios/version.scen.json");
}
