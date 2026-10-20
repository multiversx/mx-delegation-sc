# Delegation fuzzer: Go → Rust migration plan

Port of the Go fuzzer at `mx-chain-vm-go/fuzz/delegation/v0.5` into this repo's interactor,
running against the **chain simulator** instead of the in-process Go VM mock.

Primary target is **v0.5** (the `latest` crate). v0.4 / v0.3 matter for upgrade-path testing.

---

## 1. What the old fuzzer does

`fuzzDelegation_test.go` deploys the delegation contract plus an auction mock, creates
`numDelegators` delegator accounts, then runs N randomly-chosen events from a weighted
roulette, checking two on-chain invariants after most of them. At the end it forces a
wind-down and asserts three closing properties.

| File | Contents |
| --- | --- |
| `fuzzDelegation_test.go` | seed/iteration flags, the roulette weights, closing invariants |
| `fuzzDelegationExecutor.go` | scenario-step plumbing, `addNodes`/`removeNodes`, block nonce, `modifyTotalDelegationCap`, `setServiceFee`, `continueGlobalOperation`, `isBootstrapMode` |
| `fuzzDelegationInit.go` | world setup, account creation, deploy |
| `fuzzDelegationStake.go` | `stake` / `unStake` / `unBond`, stake-by-type printing |
| `fuzzDelegationRewards.go` | `validatorReward` injection, `claimRewards` |
| `fuzzDelegationInvariants.go` | `validateOwnerStakeShare`, `validateDelegationCapInvariant` |
| `fuzzDelegationDust.go` | dust cleanup loops |
| `fuzzDelegationGetters.go` | address helpers, raw balance reads |

Event weights: block nonce 5, addNodes 5, removeNodes 5, stake 5, rewards 5,
**claimRewards 20**, unStake 5, unBond 5 (listed twice, so effectively 10),
delegation cap 5, service fee 5, dust 1.

Closing invariants:

1. everyone claims; `total_rewards - sum(delegator balances) < 100`
2. everyone unStakes waiting funds, blocks advance, everyone unBonds → auction balance is 0
3. `withdrawn + total_active_stake == total_stake_added`

---

## 2. Research log — what was verified against a live simulator

Checked against `multiversx/chainsimulator:latest` on `localhost:8085`.

| Question | Answer |
| --- | --- |
| Can `set-state` write the protocol-protected `ELRONDreward` key? | **Yes.** Written and read back verbatim. `setKeyValueMap` calls `SaveKeyValue` directly, bypassing the VM's protected-key guard. |
| Reward key name | `ELRONDreward` = `core.ProtectedKeyPrefix + "reward"`, hex `454c524f4e44726577617264`, value is the big-endian cumulated total |
| Is the real auction SC present? | **Yes**, at `erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqplllst77y4l` (`…0001ffff`), with code and 10 000 EGLD |
| Does the current `config.toml` point at it? | **No** — `…plllllscktaww` (`…07ffffff`) is an empty account. Pre-existing bug, fix as part of this work. |
| Are tx signatures verified? | **No.** A transaction with an all-zero signature was accepted. `no_sig` wallets work out of the box. |
| Shards | 3 (`erd_num_shards_without_meta`) |
| Max gas per tx | 600 000 000 — the Go fuzzer's 1 000 000 000 for `addNodes` does not fit |
| Min gas price | 1 000 000 000 — **fees are real**, balance-based accounting needs rework |
| Rounds per epoch | 20 |
| Node price | `GenesisNodePrice = 2500 EGLD` |
| BLS signature checking on `stake` | mx-chain-go's own simulator tests use `MockBLSSignature = "010101"`, so the system SC does not verify it |
| Does `set-state` clobber contract code? | No — `setScDataIfNeeded` only writes non-empty fields. Empty `balance` is also preserved. |
| Does `set-state` clobber nonce? | **Yes** — Go's DTO is `*uint64`, the Rust `SetStateAccount.nonce` is a plain `u64`, so it always sends a value. Read the nonce first and pass it back. |
| Endpoint coverage | Every endpoint the fuzzer needs already exists in `latest/output/proxy.rs`, incl. `validate_owner_stake_share`, `validate_delegation_cap_invariant`, `count_dust_items_*`, `dust_cleanup_*`, and `unbond_user` returning the unbonded `BigUint` |
| Older contract builds | `v0_4_0_genesis`, `v0_4_1`, `v0_5_1` … `v0_5_10` all have prebuilt `.wasm` in `output/` |

---

## 3. Decisions taken

1. **Two configs.** `config.toml` stays as-is in role, documented as the real-chain /
   mainnet investigation config. New `test.toml` covers everything on the chain simulator:
   the fuzzer and the manual CLI actions. Commands select their config automatically.
2. **No shard restriction.** Delegator addresses are spread across all shards on purpose —
   single-shard was a limitation of the old harness, not a goal.
3. **Block counts scaled down** to something reasonable (see §7b). The Go values
   (`r.Intn(1000)`) are impractical at 20 rounds/epoch.
4. **Real auction SC**, at `erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqplllst77y4l`.
   `auction-mock` stays in the repo — the existing scenario tests need it — but the
   interactor ignores it entirely. No mock deployment, no mock config, no mock proxy.
5. **`no_sig` wallets.** Delegators are random public keys with no private key, with the
   last byte varied so they land in different shards. Shard assignment uses the existing
   `ShardConfig::compute_id`, not hand-rolled mask arithmetic (§7a).
6. **Injected rewards to start.** A more natural setup is worth investigating later; see
   §4 for what it would take.

---

## 4. Rewards: natural vs injected

**Decision: start with injection.** A more natural setup is a later investigation, and
this section records what it would involve so that investigation does not start cold.

Reward injection was necessary in the old harness because the Go VM mock had no protocol at
all. The chain simulator does have one, so natural per-epoch rewards are *mechanically*
possible — but they are a different kind of test, and they are not free.

To earn protocol rewards the delegation contract must own nodes that are actually staked at
the system auction SC *and* actually signing blocks. That requires:

- **Real BLS keypairs.** Fake keys can be staked (signature checking is off), but the
  simulator can only sign blocks with keys it holds. A fake key shuffled into the eligible
  set would stall block production. The private keys must be registered through
  `POST /simulator/add-keys` (`{"privateKeysBase64": [...]}`).
  MultiversX BLS is herumi-flavoured BLS12-381; reimplementing keygen in Rust is risky, so
  the practical route is a **committed fixture file of pre-generated keypairs**
  (private key base64 + public key hex), produced once by a small Go helper.
- **Real stake.** 2500 EGLD per node, routed through `stakeNodes` on the delegation
  contract. Fundable on the simulator, but it drags the whole node-activation subsystem in.
- **Epoch pacing.** Rewards only materialise at epoch boundaries, and a freshly staked node
  needs a few epochs to become eligible. At 20 rounds/epoch that is tens of blocks per
  reward event, versus zero for injection.
- **No amount control.** The fuzzer could no longer aim rewards at accounting edge cases
  (0, 1 wei, very large). It gets whatever the protocol pays.
- **Churn risk.** The old event mix adds and removes nodes constantly. Doing that against a
  live consensus set can destabilise the simulator.

Also worth noting: **the v0.5 Go fuzzer never stakes nodes at all.** It only calls
`addNodes` / `removeNodes`. So natural rewards are not a port of existing behaviour — they
are new scope.

Two further points in injection's favour:

- Injection is not a fake. It writes exactly the key the protocol writes, with the same
  semantics. `get_cumulated_validator_rewards()` cannot tell the difference.
- The thing under test is the contract's **reward accounting**, not the protocol's reward
  distribution. The protocol's half is already covered by mx-chain-go's own tests.

### What a later natural-rewards investigation would add

It is genuinely valuable, for two reasons: it removes the `set-state` write entirely, and
it exercises `stakeNodes` / `unStakeNodes` / `unBondNodes` / `unJail` in
`node_activation.rs`, which the Go fuzzer never touched. The interesting end state is a
hybrid — real nodes earning real rewards, *plus* injected edge-case amounts on top.

Because the real auction SC is in place from day one (decision 4), nothing has to be
rebuilt when that investigation happens. `reward_mode` is kept as a config key with a
single implemented value, `"injected"`, so adding `"natural"` later is additive.

---

## 5. Layout

```
interactor/src/
  deleg_interactor.rs            # main + CLI dispatch
  deleg_interactor_cli.rs        # extended: fuzz + per-operation actions
  deleg_interactor_config.rs     # real-chain config (config.toml)
  deleg_interactor_{admin,user,stats}.rs
  latest_proxy.rs
  auction_proxy.rs               # NEW: real auction SC (queries only, for invariants)
  ops/                           # NEW: endpoint calls, config-agnostic
    mod.rs                       #   fn stake(interactor, sc, sender, amount) -> ...
    stake.rs  rewards.rs  nodes.rs  admin.rs  dust.rs  views.rs
  test_env/                      # NEW: chain-simulator harness (test.toml)
    mod.rs
    config.rs                    # TestConfig
    accounts.rs                  # no_sig address generation, shard spread, funding
    setup.rs                     # deploy + fund + register   (fuzzDelegationInit.go)
  fuzz/                          # NEW: the fuzzer proper
    mod.rs
    roulette.rs                  # weighted choice + seeded RNG
    events.rs                    # the event table              (fuzzDelegation_test.go)
    state.rs                     # totals: staked / rewarded / withdrawn
    invariants.rs                # per-step + closing checks
    log.rs                       # action log for replay
    run.rs                       # the loop
```

The key refactor is **`ops/`**: the endpoint-calling code becomes free functions taking
`(&mut Interactor, sc_address, sender, args…)`. Both the real-chain commands and the
simulator/fuzz commands then call the same code with different arguments, instead of the
current pattern where the call is welded to `self.config`. This is also what makes "every
fuzz operation also gets a CLI action" cheap.

---

## 6. Config split

### `config.toml` — real chains

Unchanged in structure. Gains a doc header saying it is for mainnet/devnet/shadow-fork
investigation. The wrong `auction_address` gets fixed.

### `test.toml` — chain simulator (fuzzer + manual test actions)

```toml
[chain_config]
chain_type = 'simulator'
gateway_uri = 'http://localhost:8085'
# sc_address is written here by `deploy` / `fuzz-setup`, or left out for a fresh deploy

[contracts]
delegation      = '../latest/output/delegation_latest_full.wasm'
auction_address = 'erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqplllst77y4l'

[owner]
test_wallet = 'alice'

[delegators]
count = 10
# addresses are generated as no_sig keys from the run seed, spread over all shards
shard_spread = true
initial_balance = "10000000000000000000000"   # 10 000 EGLD, set via set-state
gas_budget     = "100000000000000000000"      # 100 EGLD, topped up when low

[deploy]                       # randomized per run within these bounds
service_fee_max                = 10000
owner_min_stake_share          = 0
min_stake_max                  = 1000000
n_blocks_before_unbond_max     = 20
stake_per_node                 = "2500000000000000000000"
delegation_cap_multiplier      = 4

[fuzz]
seed                 = 0       # 0 = time-based; always printed at startup
iterations           = 1000
reward_mode          = "injected"   # only value implemented; see §4
block_advance_max    = 20
invariant_check_every = 1      # 1 = Go parity; raise for speed
reward_tolerance     = 100

[fuzz.weights]
block_nonce = 5
add_nodes = 5
remove_nodes = 5
stake = 5
rewards = 5
claim_rewards = 20
unstake = 5
unbond = 10
delegation_cap = 5
service_fee = 5
dust = 1

[gas]                          # chain cap is 600M
add_nodes        = 500_000_000
unstake          = 500_000_000
unbond           =  90_000_000
continue_global  = 400_000_000
dust_cleanup     = 300_000_000

[actions]                      # defaults for the manual CLI operations
stake_amount   = "2000000000000000000"
unstake_amount = "42000000000000000000000"
```

`[actions]` replaces the constants currently hardcoded in `deleg_interactor_user.rs`
(2 EGLD stake, 42 000 EGLD unstake) and `deleg_interactor_admin.rs` (the delegation cap).

### Automatic selection

Each CLI command declares which config it belongs to:

```rust
impl InteractCliCommand {
    fn config_file(&self) -> &'static str {
        match self {
            // real-chain investigation
            Version | Global | Settings | UserFull | NumUsers | UserStake(_)
            | Upgrade | ContinueGlobal | FixUsers => "config.toml",
            // chain-simulator testing
            Fuzz(_) | FuzzSetup | Deploy | AddNodes(_) | RemoveNodes(_) | Stake(_)
            | Unstake(_) | Unbond(_) | Claim(_) | AddRewards(_) | SetServiceFee(_)
            | ModifyCap(_) | DustCleanup(_) | GenerateBlocks(_) | CheckInvariants
                => "test.toml",
        }
    }
}
```

with a global `--config <file>` escape hatch. `main` parses the CLI **before** constructing
the interactor (today it does the reverse), then loads the right file via
`Interactor::load_config_from_file`.

---

## 7. Chain-simulator mechanics

### a) Accounts — `no_sig`, spread across shards

Delegator "wallets" are random 32-byte public keys with no private key, derived
deterministically from the run seed so a run is reproducible. `WalletConfig::from_no_sig`
already supports this, and the simulator accepts zero signatures (verified).

Shard placement reuses what already exists — `ShardConfig::compute_id`, in
`multiversx_sc::chain_core::types`, which mirrors Go's `multiShardCoordinator` and is
covered by its own tests:

```rust
use multiversx_sc::chain_core::types::{ShardConfig, ShardId};

let shard: ShardId = ShardConfig::THREE_SHARDS.compute_id(&address);
```

The generator varies the last address byte and keeps the keys whose computed shard is the
one it wants, placing delegators in shards 0/1/2 round-robin for genuine cross-shard
traffic against the contract. Random 32-byte keys never collide with the SC address format,
so no filtering is needed. `ShardConfig::new(n)` covers simulators started with a different
shard count, and `test_wallets::for_shard` is available where a signing wallet in a
specific shard is wanted instead.

Funding is by `set-state` on balance (precise). `send_user_funds` exists but sends a fixed
simulator-chosen amount, so it is only useful for coarse top-ups.

**Cross-shard caveat:** calls from a delegator in shard B to a contract in shard A settle
over more blocks, and the interactor's `generate_blocks_until_tx_processed` caps at 20
blocks per transaction. This needs an early smoke test — it is the main new risk introduced
by decision 2.

### b) Block and epoch advance

`increaseBlockNonce(n)` → `interactor.generate_blocks(n)`. `generate_blocks_until_epoch` is
available for the natural-rewards mode.

The Go magnitudes do not survive the move: `numBlocksBeforeUnbond` up to 1000 and a
per-event advance of up to 1000 would mean up to 50 epoch transitions per single event.
Both become config-driven, defaulting to ~20, which keeps the deferred-payment / unbond
logic meaningfully exercised without the cost.

### c) Reward injection (`reward_mode = "injected"`)

This lives in the framework, not here:
`InteractorBase::inject_validator_reward(address, amount)`, alongside
`get_cumulated_validator_rewards` and the `VALIDATOR_REWARD_KEY` constant
(`framework/snippets/src/interactor/interactor_chain_simulator.rs`). The harness just
calls it.

What it does, per reward event:

1. Reads the account (balance, nonce) and its current `ELRONDreward` value.
2. Issues one `set-state` with `balance = old + reward`,
   `pairs = { "454c524f4e44726577617264": be_hex(cumulated + reward) }`,
   empty `code` (preserved), and **the real nonce passed back** so it is not reset to 0.

Safe as a read-modify-write because the interactor is sequential and generates blocks until
each transaction settles, so nothing is in flight against that account. It is a no-op
returning `"no-simulator"` when the interactor is not pointed at a chain simulator.

### d) Fees break Go's balance accounting

The Go fuzzer sums delegator balances to prove rewards were paid out. Here every
transaction burns a real fee, so that sum drifts. Replaced with **sink accounts**:

- Each delegator holds a gas budget, topped up by `set-state` when it drops below a
  threshold. The budget is never part of any invariant.
- After each `claimRewards`, the exact claimed amount — read from `getClaimableRewards`
  *before* the call, which is what the Go fuzzer already does — is forwarded to a
  **rewards sink** address.
- After each `unBond`, the exact amount returned by `unbond_user` is forwarded to a
  **withdraw sink** address (the Go fuzzer already does this).
- Sinks never send transactions, so their balances are exact.

Fallback if forwarding proves awkward: sum the `fee` field of every transaction result and
subtract. Less clean, kept in reserve.

### e) Gas

`addNodes` at 1 000 000 000 exceeds the 600 000 000 cap. Per-operation gas moves to
`[gas]`, and node batch sizes are capped so a batch fits in one transaction.

### f) One free win

The Go fuzzer ran its views as `scCall` steps because of its harness. Here they become
`.query()` — no transaction, no block, no fee. That removes a large fraction of the
per-event cost, and partly pays for the invariant transactions.

---

## 8. Operation mapping

| Go | Rust |
| --- | --- |
| `executeTxStep(scCall)` | `interactor.tx()…typed(DelegationFullProxy)` |
| `querySingleResult`, `delegatorQuery`, `getUserStakeOfType` | `interactor.query()` — free |
| `setBlockNonce` / `increaseBlockNonce` | `generate_blocks(n)` |
| `addRewards` (`validatorReward` step) | `set-state` on `ELRONDreward` + balance (§7c) |
| `stake` (faucet transfer, then stake) | `set-state` top-up + `stake_endpoint().egld(..)` |
| `unStake` | `unstake_endpoint(amount)` |
| `unBond` loop | same loop; `unbond_user` returns the amount; forward to withdraw sink |
| `claimRewards` | query before, call, query after (assert 0), forward to rewards sink |
| `addNodes` / `removeNodes` | `add_nodes` / `remove_nodes`, batch-capped for gas |
| `modifyDelegationCap` / `setServiceFee` | same, both return `OperationCompletionStatus` |
| `continueGlobalOperation` | already implemented in `deleg_interactor_admin.rs` — reuse |
| `isBootstrapMode` | `.query()` instead of a transaction |
| `dustCleanup` loops | `count_dust_items_*` queries + `dust_cleanup_*` calls |
| `blsKey(i)` / `blsSignature(i)` | 96-byte / 48-byte padded patterns (`BLS_KEY_BYTE_LENGTH`, `BLS_SIGNATURE_BYTE_LENGTH`) |
| `roulette.RandomChoice` | `fuzz/roulette.rs`, weights from `[fuzz.weights]` |
| `saveGeneratedScenario` | action log + optional interactor scenario trace (§10) |

---

## 9. Invariants

**Per step** (Go parity): `validateOwnerStakeShare` and `validateDelegationCapInvariant`.
Both are transactions that fail when violated, so they cost two transactions per event —
hence `invariant_check_every`.

**Closing**, ported from `fuzzDelegation_test.go`:

1. Everyone claims → `rewards_sink ≈ total_rewards` within `reward_tolerance`.
   In `natural` mode the reference value is the contract's own `getTotalCumulatedRewards`
   rather than a locally tracked sum.
2. Everyone unStakes waiting funds, blocks advance, everyone unBonds.
3. `withdraw_sink + total_active_stake == total_stake_added`.
4. `getClaimableRewards == 0` after every claim.

**Adapted:** Go's "auction has no balance" check cannot survive the move to the real auction
SC, which holds its own funds and other validators' stake. It is replaced by *"the
delegation contract has no stake left at the auction SC"*, queried per-contract-address.
With injected rewards and no node staking, nothing is ever sent to the auction anyway, so
this check is cheap insurance rather than a live assertion — it only bites once nodes are
actually staked.

---

## 10. Determinism and replay

One seeded `StdRng`, seed printed first and settable with `--seed`, as in Go.

Unlike the Go VM mock, the simulator adds jitter: `generate_blocks_until_tx_processed`
produces a variable number of blocks, so block nonces — and therefore unbond timing — do
not reproduce from the seed alone. Two mitigations:

- **Action log** (JSONL): every operation with its concrete parameters, plus a
  `--replay <file>` mode that re-executes it verbatim. This is the reliable path.
- **Scenario trace**: `Interactor::with_tracer(path)` already emits
  `interactor_trace.scen.json`, the direct analogue of Go's `fuzz_gen.scen.json`, replayable
  in the normal scenario runners. Caveat: `set-state` reward injections are invisible to the
  tracer and would have to be spliced in as `validatorReward` steps. Stretch goal.

---

## 11. CLI actions

Every fuzz operation also gets a manual command, all reading `test.toml`:

```
fuzz [--seed N] [--iterations N]     run the fuzzer
fuzz-setup                           deploy + fund + register, leave a usable environment
deploy                               deploy the delegation contract
add-nodes <n> / remove-nodes <n>
stake <delegator> [amount]
unstake <delegator> [amount]
unbond <delegator>
claim <delegator>
add-rewards <amount>                 set-state injection (simulator only)
set-service-fee <value>
modify-cap <amount>
dust-cleanup <limit>
generate-blocks <n>
check-invariants
```

Later, with natural rewards: `stake-nodes`, `unstake-nodes`, `unbond-nodes`, `add-bls-keys`.

Existing real-chain commands (`version`, `global`, `settings`, `user-full`, `num-users`,
`user-stake`, `upgrade`, `continue-global`, `fix-users`) keep working against `config.toml`.

---

## 12. Older versions and upgrades

v0.3 and v0.4 differ in their init and event sets — v0.4 has `stakeGenesis` /
`activateGenesis`, v0.3 has `enableAutoActivation` / `stakeAllAvailable`, neither of which
exists in v0.5. Porting those event sets wholesale is low value.

The high-value piece is **migration**: a `[migration]` section with `start_version` and
`upgrade_at_event`, which deploys an older full contract, fuzzes against it, upgrades to
`latest_update` mid-run, then continues fuzzing and runs the same closing invariants across
the upgrade boundary. Every old `.wasm` is already built, so this is config plus one new
event type.

---

## 13. Phases

| Phase | Deliverable | Gate |
| --- | --- | --- |
| 0 | `test.toml` + `TestConfig`, config auto-selection, `ops/` refactor, reward injection | Inject a reward into a freshly deployed contract and watch `getTotalCumulatedRewards` move |
| 1 | `test_env/`: deploy, generate `no_sig` delegators across shards, fund them, sinks. `fuzz-setup` | A cross-shard `stake` from a shard-2 delegator settles within the block budget |
| 2 | Each operation in `ops/` + its CLI action, with Go-equivalent logging | All commands work by hand |
| 3 | Roulette, seeded event loop, per-step invariants, action log | A 100-iteration run completes clean |
| 4 | Closing invariants, final report, `--replay` | A 1000-iteration run completes; throughput measured |
| 5 | Migration: older versions + mid-run upgrade events | |
| 6 | Optional: scenario-trace export, CI profile | |

Deferred, not scheduled: `reward_mode = "natural"` (§4) — BLS keypair fixtures,
`/simulator/add-keys`, node staking events.

Phase 0 and Phase 1 are the risk gates — reward injection and cross-shard latency
respectively. Everything after is mechanical.

Throughput gets measured at the end of Phase 4. If a 1000-iteration run is too slow, the
levers are `invariant_check_every`, batching independent transactions through the existing
`homogenous_tx_buffer`, and lowering `block_advance_max`.

---

## 14. Deferred questions

Nothing blocking. Parked until the natural-rewards investigation (§4) actually starts:

- Where the BLS keypair fixtures should live, and whether generating them with a small Go
  helper (committed output) is acceptable.
- How node add/remove churn interacts with a live consensus set without wedging the
  simulator.
