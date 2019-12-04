#![cfg_attr(not(feature = "std"), no_std)]
// The above line is needed to compile the Wasm binaries.

// Importing crates declared in the cargo.toml file.
use codec::{Decode, Encode};
use primitives::H256;
use structures::Real;
use support::{decl_module, decl_storage, dispatch::Result, StorageMap};

// Importing the rest of the files in this crate.
mod burn;
mod mint;
mod transfer;
use burn::*;
use mint::*;
use transfer::*;

// This module's configuration trait.
pub trait Trait: system::Trait {}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as AssetsStorage {
        pub Balances get(fn balances): map (u32, H256) => Real;
        pub TotalSupply get(fn total_supply): map u32 => Real;
    }
}

// This module's dispatchable functions.
decl_module! {
    // The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        pub fn dispatch_transfer(origin,from_address: H256, to_address: H256, asset_id: u32, amount: Real) -> Result {
            // Call corresponding internal function.
            Self::transfer(from_address, to_address, asset_id, amount)?;

            // Return Ok if successful.
            Ok(())
        }

        pub fn dispatch_mint(origin, to_address: H256, asset_id: u32, amount: Real) -> Result {
            // Call corresponding internal function.
            Self::mint(to_address, asset_id, amount)?;

            // Return Ok if successful.
            Ok(())
        }

        pub fn dispatch_burn(origin,from_address: H256, asset_id: u32, amount: Real) -> Result {
            // Call corresponding internal function.
            Self::burn(from_address, asset_id, amount)?;

            // Return Ok if successful.
            Ok(())
        }
    }
}