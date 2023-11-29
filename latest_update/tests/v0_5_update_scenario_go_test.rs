use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    ScenarioWorld::vm_go()
}

#[test]
fn genesis_addr_fix_go() {
    world().run("scenarios/genesis_addr_fix.scen.json");
}

#[test]
fn version_go() {
    world().run("scenarios/version.scen.json");
}
