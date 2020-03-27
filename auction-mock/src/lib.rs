
#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![allow(unused_attributes)]


// global contract variables
static STAKE_KEY:     [u8; 32] = [0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];
static NR_NODES__KEY: [u8; 32] = [0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];

static BLS_PREFIX:       u8 = 0x0b;
static SIGNATURE_PREFIX: u8 = 0x0c;

// constructs keys for user data
fn node_data_key(prefix: u8, node_id: usize) -> StorageKey {
    let mut key = [0u8; 32];
    key[0] = prefix;
    elrond_wasm::serialize_i64(&mut key[28..32], node_id as i64);
    key.into()
}

#[elrond_wasm_derive::contract(AuctionMockImpl)]
pub trait AuctionMock {

    fn init(&self) {
    }

    #[payable]
    fn stake(&self,
            nr_nodes: i64,
            #[multi(2*nr_nodes)] bls_keys_signatures: Vec<Vec<u8>>,
            #[payment] payment: &BigUint) -> Result<(), &str> {

        self.storage_store_big_uint(&STAKE_KEY.into(), payment);
        self.storage_store_i64(&NR_NODES__KEY.into(), nr_nodes);

        let nr_nodes_usize = nr_nodes as usize;
        if 2*nr_nodes_usize != bls_keys_signatures.len() {
            return Err("bad bls_keys_signatures length")
        }

        for n in 0..nr_nodes_usize {
            self.storage_store(&node_data_key(BLS_PREFIX, n), &bls_keys_signatures[2*n]);
            self.storage_store(&node_data_key(SIGNATURE_PREFIX, n), &bls_keys_signatures[2*n+1])
        }

        Ok(())
    }
}
