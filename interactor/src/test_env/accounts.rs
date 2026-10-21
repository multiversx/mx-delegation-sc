//! The accounts a fuzz run operates with: the delegators that act on the contract, and
//! the sinks that collect whatever leaves it.
//!
//! Delegators are plain public keys with no private key. The chain simulator accepts
//! all-zero signatures, so there is nothing to generate, store or protect - an address is
//! enough. They are derived from a seed, so a run is reproducible without writing
//! anything to disk, and spread across every shard on purpose: the contract should see
//! genuine cross-shard traffic, which the original Go fuzzer's single-world VM could not
//! produce.

use multiversx_sc::chain_core::types::{ShardConfig, ShardId};
use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::sdk::chain_core::std::Bech32Hrp;
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::test_env::DelegatorsConfig;

/// Why the sinks exist: on the chain simulator every transaction burns a real fee, so the
/// Go fuzzer's trick of summing delegator balances to prove rewards were paid out does not
/// survive the move. Instead, whatever a delegator receives is forwarded to a sink
/// immediately. Sinks never send a transaction, so their balances stay exact.
pub struct TestAccounts {
    /// Delegator 0 is the contract owner, as in the Go fuzzer; the rest are generated.
    delegators: Vec<Bech32Address>,

    /// Collects claimed rewards.
    pub rewards_sink: Bech32Address,

    /// Collects unbonded stake.
    pub withdraw_sink: Bech32Address,
}

impl TestAccounts {
    pub fn generate(
        owner: Bech32Address,
        config: &DelegatorsConfig,
        shard_config: &ShardConfig,
        hrp: Bech32Hrp,
    ) -> Self {
        let mut rng = StdRng::seed_from_u64(config.seed);

        let mut delegators = Vec::with_capacity(config.count + 1);
        delegators.push(owner);
        for index in 0..config.count {
            let shard = ShardId::from((index as u32) % shard_config.number_of_shards);
            delegators.push(generate_in_shard(&mut rng, shard_config, shard, hrp));
        }

        // The sinks come from the same stream, so they move with the seed too. Their shard
        // does not matter - nothing is ever sent from them.
        let rewards_sink = generate_in_shard(&mut rng, shard_config, ShardId::from(0), hrp);
        let withdraw_sink = generate_in_shard(&mut rng, shard_config, ShardId::from(0), hrp);

        TestAccounts {
            delegators,
            rewards_sink,
            withdraw_sink,
        }
    }

    /// Delegator by index, 0 being the owner.
    pub fn delegator(&self, index: usize) -> &Bech32Address {
        self.delegators.get(index).unwrap_or_else(|| {
            panic!(
                "delegator {index} does not exist; there are {} (0 is the owner)",
                self.delegators.len()
            )
        })
    }

    /// The generated delegators, without the owner. These are the ones the harness
    /// creates state for; the owner is a real wallet that already exists.
    pub fn generated_delegators(&self) -> &[Bech32Address] {
        &self.delegators[1..]
    }

    pub fn sinks(&self) -> [&Bech32Address; 2] {
        [&self.rewards_sink, &self.withdraw_sink]
    }
}

/// Draws an address in a given shard.
///
/// Shard assignment depends only on the trailing bytes, so the last one is advanced until
/// it lands in the wanted shard. Starting from the drawn byte rather than from zero keeps
/// the address looking like a key rather than a crafted value. Every shard is reachable
/// from some last byte, so this always terminates.
fn generate_in_shard(
    rng: &mut StdRng,
    shard_config: &ShardConfig,
    shard: ShardId,
    hrp: Bech32Hrp,
) -> Bech32Address {
    let mut bytes = [0u8; 32];
    rng.fill_bytes(&mut bytes);

    let first_candidate = bytes[31];
    for offset in 0..=u8::MAX {
        bytes[31] = first_candidate.wrapping_add(offset);
        let address = Address::new(bytes);
        if shard_config.compute_id(&address) == shard {
            return address.to_bech32(hrp);
        }
    }

    unreachable!("every shard is reachable from some last address byte")
}
