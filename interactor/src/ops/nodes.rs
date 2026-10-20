use multiversx_sc_snippets::imports::*;
use node_storage::types::{bls_key::BLSKey, bls_sig::BLSSignature};

use crate::{latest_proxy, ops::DelegationOps};

/// `NodeState::Inactive` as the contract encodes it: registered here, but not staked at
/// the auction contract. Only nodes in this state can be removed.
pub const NODE_STATE_INACTIVE: u8 = 0;

impl DelegationOps<'_> {
    /// How many node slots the contract has handed out.
    ///
    /// Monotonic: removing a node marks it `Removed` and leaves the count alone, so this
    /// doubles as the next free key index.
    pub async fn num_nodes(&mut self) -> usize {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .num_nodes()
            .returns(ReturnsResult)
            .run()
            .await
    }

    /// The state of every node, in registration order, so entry `i` belongs to the node
    /// registered with key index `i`.
    pub async fn node_states(&mut self) -> Vec<u8> {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_all_node_states()
            .returns(ReturnsResult)
            .run()
            .await
            .into_iter()
            .map(|entry| entry.into_tuple().1)
            .collect()
    }

    /// Registers `count` nodes, numbered on from the nodes already there.
    ///
    /// Only the owner may do this. Registration is bookkeeping - nothing is sent to the
    /// auction contract until the nodes are staked.
    pub async fn add_nodes(&mut self, owner: &Bech32Address, first_index: usize, count: usize) {
        let mut bls_keys_signatures = MultiValueEncoded::new();
        for index in first_index..first_index + count {
            bls_keys_signatures.push((bls_key(index), bls_signature(index)).into());
        }

        self.interactor
            .tx()
            .from(owner)
            .to(self.sc_address)
            .gas(self.gas.add_nodes)
            .typed(latest_proxy::DelegationFullProxy)
            .add_nodes(bls_keys_signatures)
            .run()
            .await;
    }

    /// Removes nodes by index. Fails unless every one of them is inactive, which is why
    /// the caller gets the error rather than a panic.
    pub async fn remove_nodes(
        &mut self,
        owner: &Bech32Address,
        indexes: &[usize],
    ) -> Result<(), String> {
        let mut bls_keys = MultiValueEncoded::new();
        for &index in indexes {
            bls_keys.push(bls_key(index));
        }

        self.interactor
            .tx()
            .from(owner)
            .to(self.sc_address)
            .gas(self.gas.remove_nodes)
            .typed(latest_proxy::DelegationFullProxy)
            .remove_nodes(bls_keys)
            .returns(ReturnsHandledOrError::new())
            .run()
            .await
            .map_err(|err| err.message)
    }
}

/// Stand-in BLS keys, in the same shape the Go fuzzer used, so generated scenarios stay
/// recognisable between the two.
///
/// Nothing verifies these: the contract only stores them, and the system auction contract
/// does not check signatures. They would have to become real key pairs the moment the
/// harness starts staking nodes for real rewards.
pub fn bls_key(index: usize) -> BLSKey<StaticApi> {
    BLSKey {
        bytes: ManagedByteArray::new_from_bytes(&dot_padded(format!("bls key {index:5} "))),
    }
}

pub fn bls_signature(index: usize) -> BLSSignature<StaticApi> {
    BLSSignature {
        bytes: ManagedByteArray::new_from_bytes(&dot_padded(format!(
            "bls key signature {index:5} "
        ))),
    }
}

/// Fills a fixed-size buffer with `prefix`, padded out with dots.
fn dot_padded<const N: usize>(prefix: String) -> [u8; N] {
    let prefix = prefix.into_bytes();
    assert!(
        prefix.len() <= N,
        "BLS placeholder prefix is {} bytes, does not fit in {N}",
        prefix.len()
    );

    let mut bytes = [b'.'; N];
    bytes[..prefix.len()].copy_from_slice(&prefix);
    bytes
}
