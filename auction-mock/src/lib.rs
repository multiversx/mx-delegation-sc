
#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![allow(unused_attributes)]


// global contract variables
static STAKE_KEY: [u8; 32] = [0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];

#[elrond_wasm_derive::contract(AuctionMockImpl)]
pub trait AuctionMock {

    fn init(&self) {
    }

    #[payable(payment)]
    fn stake(&self, payment: &BigUint) {
        self.storage_store_big_uint(&STAKE_KEY.into(), payment);
    }
}
