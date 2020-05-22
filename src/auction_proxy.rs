
use elrond_wasm::{Box, Vec, String};
use elrond_wasm::{H256, Address, StorageKey, ErrorMessage};
use elrond_wasm::{ContractHookApi, ContractIOApi, BigIntApi, BigUintApi, AsyncCallResult, AsyncCallError};
use elrond_wasm::err_msg;
use elrond_wasm::serde as serde;
use core::ops::{Add, Sub, Mul, Div, Rem};
use core::ops::{AddAssign, SubAssign, MulAssign, DivAssign, RemAssign};
use core::ops::{BitAnd, BitOr, BitXor, Shr, Shl};
use core::ops::{BitAndAssign, BitOrAssign, BitXorAssign, ShrAssign, ShlAssign};

use super::*;
use super::bls_key::*;


