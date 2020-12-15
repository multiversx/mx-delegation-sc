#![no_std]
#![allow(unused_attributes)]
#![allow(clippy::string_lit_as_bytes)]

// auxiliaries
pub mod types;

// modules
pub mod fund_module;
pub mod fund_transf_module;
pub mod fund_view_module;
pub mod user_data;

#[macro_use]
extern crate elrond_wasm;

imports!();
