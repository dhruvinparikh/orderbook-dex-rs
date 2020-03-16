//! # DEX module
//!

#![cfg_attr(not(feature = "std"), no_std)]
// The above line is needed to compile the Wasm binaries.

// Importing crates declared in the cargo.toml file.
use core::convert::{TryFrom, TryInto};
use primitives::U256;
use rstd::if_std;
use rstd::{ops::Not, prelude::*, result};
use sp_runtime::traits::{Bounded, CheckedSub, Hash, Member, AtLeast32Bit, Zero};
use sp_runtime::RuntimeDebug;
use support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{Get, Randomness},
    Parameter, StorageMap, StorageValue,
};

use byteorder::{ByteOrder, LittleEndian};
use codec::{Decode, Encode, EncodeLike};
use system::ensure_signed;

mod create_order;
mod exchange;
mod exchange_pair;
mod market;
mod match_order;
mod price;
mod types;

pub use exchange::*;
pub use types::*;
