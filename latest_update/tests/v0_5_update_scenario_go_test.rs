#[test]
fn genesis_addr_fix_go() {
    multiversx_sc_scenario::run_go("scenarios/genesis_addr_fix.scen.json");
}

#[test]
fn version_go() {
    multiversx_sc_scenario::run_go("scenarios/version.scen.json");
}
