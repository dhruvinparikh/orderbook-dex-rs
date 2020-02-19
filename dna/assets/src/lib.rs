//! # Assets module
//!

#![cfg_attr(not(feature = "std"), no_std)]
// The above line is needed to compile the Wasm binaries.

// Importing crates declared in the cargo.toml file.
use codec::{Decode, Encode};
// use core::ops::{Add, AddAssign, Sub, SubAssign};
use rstd::prelude::*;
use sp_runtime::traits::{Bounded, Hash};
use support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    traits::Randomness, StorageMap, StorageValue,
};
use system::ensure_signed;

// Importing the rest of the files in this crate.
pub mod burn;
pub mod freeze;
pub mod mint;
pub mod transfer;
pub mod unfreeze;

/// The scale factor (must be positive).
// const SF: i128 = 1000000000;

/// This struct implements the DNAi64 data type. It is a tuple containing a single Option of
/// an i64.
// #[derive(Copy, Clone, Decode, Debug, Encode, Default, PartialEq, Eq, PartialOrd, Ord)]
// pub struct DNAi64(pub Option<i64>);

// impl DNAi64 {
//     /// Transforms an i64 into a DNAi64. It scales the input by the scale factor.
//     pub fn from(x: i64) -> DNAi64 {
//         DNAi64(x.checked_mul(SF as i64))
//     }
// }

/// Calculates the sum of two DNAi64s. If any of the inputs is 'None' (or the result over/underflows),
/// it returns 'None'. It does operator overloading for the symbol '+'.
// impl Add for DNAi64 {
//     type Output = DNAi64;

//     fn add(self, rhs: DNAi64) -> DNAi64 {
//         if self.0.is_some() && rhs.0.is_some() {
//             DNAi64(self.0.unwrap().checked_add(rhs.0.unwrap()))
//         } else {
//             DNAi64(None)
//         }
//     }
// }

/// Implements the addition assignment operator +=. Follows the same rules as the
/// addition operator.
// impl AddAssign for DNAi64 {
//     fn add_assign(&mut self, rhs: Self) {
//         *self = *self + rhs;
//     }
// }

/// Calculates the subtraction of two DNAi64s. If any of the inputs is 'None' (or the result
/// over/underflows), it returns 'None'. It does operator overloading for the symbol '-'.
// impl Sub for DNAi64 {
//     type Output = DNAi64;

//     fn sub(self, rhs: DNAi64) -> DNAi64 {
//         if self.0.is_some() && rhs.0.is_some() {
//             DNAi64(self.0.unwrap().checked_sub(rhs.0.unwrap()))
//         } else {
//             DNAi64(None)
//         }
//     }
// }

/// Implements the subtraction assignment operator -=. Follows the same rules as the
/// subtraction operator.
// impl SubAssign for DNAi64 {
//     fn sub_assign(&mut self, rhs: Self) {
//         *self = *self - rhs;
//     }
// }

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Asset<Hash, Balance> {
    pub hash: Hash,
    pub symbol: Vec<u8>,
    pub total_supply: Balance,
}

pub trait Trait: balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's configuration trait.
// pub trait Trait: system::Trait {}

decl_error! {
    /// Error for the asset module.
    pub enum Error for Module<T: Trait> {
        /// There is no match asset
        NoMatchingAsset,
        /// The balance is not enough
        BalanceNotEnough,
        /// Amount overflow
        AmountOverflow,
        /// Sender does not have asset
        SenderHaveNoAsset,
        /// Total supply cannot be negative
        NegativeAmount,
    }
}

decl_event!(
	pub enum Event<T> 
    where 
        <T as system::Trait>::AccountId,
        <T as system::Trait>::Hash,
        <T as balances::Trait>::Balance,
    {
		Issued(AccountId, Hash, Balance),
        Transfered(AccountId, AccountId, Hash, Balance),
        Burned(AccountId,Hash,Balance),
        Freezed(AccountId, Hash, Balance),
        UnFreezed(AccountId, Hash, Balance),
	}
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as AssetsStorage {
        pub Assets get(asset): map T::Hash => Option<Asset<T::Hash, T::Balance>>;
        pub Owners get(owner): map T::Hash => Option<T::AccountId>;
        pub BalanceOf get(balance_of): map (T::AccountId, T::Hash) => T::Balance;
        pub FreeBalanceOf get(free_balance_of): map (T::AccountId, T::Hash) => T::Balance;
        pub FreezedBalanceOf get(freezed_balance_of): map (T::AccountId, T::Hash) => T::Balance;

        pub OwnedAssets get(owned_asset): map (T::AccountId, u64) => Option<T::Hash>;
        pub OwnedAssetsIndex get(owned_asset_index): map T::AccountId => u64;

        Nonce: u64;
    }
}

// This module's dispatchable functions.
decl_module! {
    // The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        type Error = Error<T>;

        pub fn deposit(_origin, asset_hash: T::Hash, to: T::AccountId, amount: T::Balance) -> DispatchResult {
            let sender = ensure_signed(_origin)?;

            // Call corresponding internal function.
            Self::transfer(sender.clone(), asset_hash, to.clone(), amount)?;

            // Return Ok if successful.
            Ok(())
        }

        pub fn issue(_origin, symbol: Vec<u8>, total_supply: T::Balance) -> DispatchResult {
            // Call corresponding internal function.
            Self::mint(_origin, symbol,total_supply)?;

            // Return Ok if successful.
            Ok(())
        }

        pub fn destroy(_origin, asset_hash: T::Hash, amount: T::Balance) -> DispatchResult {
            let sender = ensure_signed(_origin)?;

            // Call corresponding internal function.
            Self::burn(sender,asset_hash,amount)?;

            // Return Ok if successful.
            Ok(())
        }

        /// # Provide info to freeze funds
        /// * `_origin` - signer
        /// * `hash` - asset hash
        /// * `amount` - amount to freeze
        pub fn do_freeze(_origin, hash: T::Hash, amount: T::Balance) -> DispatchResult {
             let sender = ensure_signed(_origin)?;

             // call corresponding internal function
             Self::freeze(sender, hash, amount)?;

             // Return Ok if successful.
             Ok(())
        }

        /// # Provide info to unfreeze funds
        /// * `_origin` - signer
        /// * `hash` - asset hash
        /// * `amount` - amount to freeze
        pub fn do_unfreeze(_origin, hash: T::Hash, amount: T::Balance) -> DispatchResult {
            let sender = ensure_signed(_origin)?;

            // call corresponding internal function
            Self::unfreeze(sender, hash, amount)?;

             // Return Ok if successful.
             Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn ensure_free_balance(
        sender: T::AccountId,
        hash: T::Hash,
        amount: T::Balance,
    ) -> DispatchResult {
        let asset = Self::asset(hash);
        ensure!(asset.is_some(), Error::<T>::NoMatchingAsset);

        ensure!(
            FreeBalanceOf::<T>::exists((sender.clone(), hash.clone())),
            Error::<T>::SenderHaveNoAsset
        );

        let free_amount = Self::free_balance_of((sender.clone(), hash.clone()));
        ensure!(free_amount >= amount, Error::<T>::BalanceNotEnough);

        Ok(())
    }
}
