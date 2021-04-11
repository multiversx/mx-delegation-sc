#![no_std]
#![allow(unused_attributes)]
#![allow(clippy::string_lit_as_bytes)]

// auxiliaries
pub mod types;

// modules
pub mod node_config;

#[macro_use]
extern crate elrond_wasm;

elrond_wasm::imports!();
