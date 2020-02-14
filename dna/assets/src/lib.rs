//! # Assets module
//!

#![cfg_attr(not(feature = "std"), no_std)]
// The above line is needed to compile the Wasm binaries.

// Importing crates declared in the cargo.toml file.
use codec::{Decode, Encode};
use core::ops::{Add, AddAssign, Sub, SubAssign};
use primitives::H256;
use rstd::prelude::*;
use support::{decl_module, decl_storage, dispatch::DispatchResult, StorageMap};

// Importing the rest of the files in this crate.
mod burn;
mod mint;
mod transfer;

/// The scale factor (must be positive).
const SF: i128 = 1000000000;

/// This struct implements the DNAi64 data type. It is a tuple containing a single Option of
/// an i64.
#[derive(Copy, Clone, Decode, Debug, Encode, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct DNAi64(pub Option<i64>);

impl DNAi64 {
    /// Transforms an i64 into a DNAi64. It scales the input by the scale factor.
    pub fn from(x: i64) -> DNAi64 {
        DNAi64(x.checked_mul(SF as i64))
    }
}

/// Calculates the sum of two DNAi64s. If any of the inputs is 'None' (or the result over/underflows),
/// it returns 'None'. It does operator overloading for the symbol '+'.
impl Add for DNAi64 {
    type Output = DNAi64;

    fn add(self, rhs: DNAi64) -> DNAi64 {
        if self.0.is_some() && rhs.0.is_some() {
            DNAi64(self.0.unwrap().checked_add(rhs.0.unwrap()))
        } else {
            DNAi64(None)
        }
    }
}

/// Implements the addition assignment operator +=. Follows the same rules as the
/// addition operator.
impl AddAssign for DNAi64 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

/// Calculates the subtraction of two DNAi64s. If any of the inputs is 'None' (or the result
/// over/underflows), it returns 'None'. It does operator overloading for the symbol '-'.
impl Sub for DNAi64 {
    type Output = DNAi64;

    fn sub(self, rhs: DNAi64) -> DNAi64 {
        if self.0.is_some() && rhs.0.is_some() {
            DNAi64(self.0.unwrap().checked_sub(rhs.0.unwrap()))
        } else {
            DNAi64(None)
        }
    }
}

/// Implements the subtraction assignment operator -=. Follows the same rules as the
/// subtraction operator.
impl SubAssign for DNAi64 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

// This module's configuration trait.
pub trait Trait: system::Trait {}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as AssetsStorage {
        pub Balances get(fn balances): map (u32, H256) => DNAi64;
        pub TotalSupply get(fn total_supply): map u32 => DNAi64;
        pub Tokens get(fn tokens): map u32 => DNAi64;
    }
}

// This module's dispatchable functions.
decl_module! {
    // The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        pub fn deposit(_origin,from_address: H256, to_address: H256, asset_id: u32, amount: DNAi64) -> DispatchResult {
            // Call corresponding internal function.
            Self::transfer(from_address, to_address, asset_id, amount)?;

            // Return Ok if successful.
            Ok(())
        }

        pub fn issue(_origin, to_address: H256, asset_id: u32, amount: DNAi64, name: u32) -> DispatchResult {
            // Call corresponding internal function.
            Self::mint(to_address, asset_id, amount, name)?;

            // Return Ok if successful.
            Ok(())
        }

        pub fn destroy(_origin,from_address: H256, asset_id: u32, amount: DNAi64) -> DispatchResult {
            // Call corresponding internal function.
            Self::burn(from_address, asset_id, amount)?;

            // Return Ok if successful.
            Ok(())
        }

        /// # Provide info to freeze funds of the order maker
        /// * `_origin` - signer
        /// * `asset_id` - asset id
        /// * `amount` - amount to freeze
        pub fn freeze(_origin, asset_id : u32, amount:Balance) -> DispatchResult {
            let sender = ensure_signed(_origin)?;

             // call corresponding internal function

             // Return Ok if successful.
             Ok(())
        }

        /// # Provide info to unfreeze funds of the order maker
        /// * `_origin` - signer
        /// * `asset_id` - asset id
        /// * `amount` - amount to freeze
        pub fn unfreeze(_origin, asset_id : u32, amount:Balance) -> DispatchResult {
            let sender = ensure_signed(_origin)?;

             // call corresponding internal function

             // Return Ok if successful.
             Ok(())
        }
    }
}
