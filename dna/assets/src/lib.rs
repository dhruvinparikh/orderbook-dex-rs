//! # Assets module
//!

#![cfg_attr(not(feature = "std"), no_std)]
// The above line is needed to compile the Wasm binaries.

// Importing crates declared in the cargo.toml file.
use codec::{Decode, Encode};
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use primitives::H256;
use rstd::fmt::Debug;
use rstd::prelude::*;
use support::{decl_module, decl_storage, dispatch::DispatchResult, StorageMap};
use system::ensure_signed;

// Importing the rest of the files in this crate.
mod burn;
mod construct;
mod mint;
mod transfer;

/// The scale factor (must be positive).
const SF: i128 = 1000000000;

// The maximum and minimum values supported by i64, as a i128. They are used for over/underflow
// checks in multiplication and division.
const MAX: i128 = i64::max_value() as i128;
const MIN: i128 = i64::min_value() as i128;

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

/// Calculates the division of two DNAi64s. If any of the inputs is 'None' (or the result
/// over/underflows), it returns 'None'. It does operator overloading for the symbol '/'.
impl Div for DNAi64 {
    type Output = DNAi64;

    fn div(self, rhs: DNAi64) -> DNAi64 {
        if self.0.is_some() && rhs.0.is_some() {
            // Casting onto larger type to prevent overflow in the intermediate calculations.
            let a: i128 = self.0.unwrap() as i128;
            let b: i128 = rhs.0.unwrap() as i128;

            // Checking for division by zero.
            if b == 0 {
                return DNAi64(None);
            }

            // Multiplying the dividend by the scale factor.
            let mut c = a * SF;

            // Calculating the remainder.
            let r = c % b;

            // Dividing by the divisor.
            c /= b;

            // Rounding depending on the remainder. It uses the 'round half away from zero' method.
            if 2 * r.abs() >= b.abs() {
                //We can't use c.signum because c may be zero.
                c += a.signum() * b.signum();
            }

            // Verifying if it over/underflows and then returning the appropriate answer.
            if c < MIN || c > MAX {
                DNAi64(None)
            } else {
                DNAi64(Some(c as i64))
            }
        } else {
            DNAi64(None)
        }
    }
}

/// Implements the division assignment operator /=. Follows the same rules as the
/// division operator.
impl DivAssign for DNAi64 {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

/// Calculates the multiplication of two DNAi64s. If any of the inputs is 'None' (or the result
/// over/underflows), it returns 'None'. It does operator overloading for the symbol '*'.
impl Mul for DNAi64 {
    type Output = DNAi64;

    fn mul(self, rhs: DNAi64) -> DNAi64 {
        if self.0.is_some() && rhs.0.is_some() {
            // Casting onto larger type to prevent overflow in the intermediate calculations.
            let a: i128 = self.0.unwrap() as i128;
            let b: i128 = rhs.0.unwrap() as i128;

            // Multiplying both numbers.
            let mut c = a * b;

            // Calculating the remainder.
            let r = c % SF;

            // Dividing by the scale factor.
            c /= SF;

            // Rounding depending on the remainder. It uses the 'round half away from zero' method.
            if 2 * r.abs() >= SF {
                //We can't use c.signum because c may be zero.
                c += a.signum() * b.signum();
            }

            // Verifying if it over/underflows and then returning the appropriate answer.
            if c < MIN || c > MAX {
                DNAi64(None)
            } else {
                DNAi64(Some(c as i64))
            }
        } else {
            DNAi64(None)
        }
    }
}

/// Implements the multiplication assignment operator *=. Follows the same rules as the
/// multiplication operator.
impl MulAssign for DNAi64 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

// This module's configuration trait.
pub trait Trait: system::Trait {}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as AssetsStorage {
        pub Balances get(fn balances): map (u32, T::AccountId) => DNAi64;
        pub TotalSupply get(fn total_supply): map u32 => DNAi64;
        pub Tokens get(fn tokens): map u32 => (u32,u32,DNAi64);
        pub Decimals get(fn decimals): map u32 => DNAi64;
        pub OwnerOf get(fn owner): map u32 => T::AccountId;
    }
}

// This module's dispatchable functions.
decl_module! {
    // The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        pub fn deposit(_origin, to_address: T::AccountId, asset_id: u32, amount: DNAi64) -> DispatchResult {
            let sender = ensure_signed(_origin)?;

            // Call corresponding internal function.
            Self::transfer(sender, to_address, asset_id, amount)?;

            // Return Ok if successful.
            Ok(())
        }

        pub fn issue(_origin, to_address: T::AccountId, asset_id: u32, amount: DNAi64) -> DispatchResult {
            let sender = ensure_signed(_origin)?;

            // Call corresponding internal function.
            Self::mint(sender,to_address, asset_id, amount)?;

            // Return Ok if successful.
            Ok(())
        }

        /// # Provide info to create an asset
        /// * `_origin` - Owner of the asset
        /// * `name` - Name of the asset
        /// * `asset_id` - ID of the asset
        /// * `decimal` - Decimals of the asset
        pub fn create(_origin, name:u32, asset_id: u32, decimals:DNAi64, symbol:u32) -> DispatchResult {
            let sender = ensure_signed(_origin)?;

            // Call corresponding internal function.
            Self::construct(sender,name, asset_id,decimals,symbol)?;

             // Return Ok if successful.
             Ok(())
        }

        pub fn destroy(_origin,from_address: T::AccountId, asset_id: u32, amount: DNAi64) -> DispatchResult {
            let sender = ensure_signed(_origin)?;

            // Call corresponding internal function.
            Self::burn(sender,from_address, asset_id, amount)?;

            // Return Ok if successful.
            Ok(())
        }
    }
}
