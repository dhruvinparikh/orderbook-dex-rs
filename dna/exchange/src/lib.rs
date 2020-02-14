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


// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as ExchangeStorage {
        ///	TradePairHash => TradePair
		TradePairs get(fn trade_pair): map T::Hash => Option<TradePair<T>>;

		/// OrderHash => Order
		Orders get(fn order): map T::Hash => Option<LimitOrder<T>>;

		/// TradeHash => Trade
		Trades get(fn trade): map T::Hash => Option<Trade<T>>;

		Nonce: u64;
    }
}


// This module's dispatchable functions.
decl_module! {
    // The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        /// # Provide info to create an order limit
        /// * `_origin` - signer
        /// * `base` - hash/asset_id of base asset
        /// * `quote` - hash/asset_id of quote asset
        /// * `price` - price per unit of the base unit
        /// * `sell_amount` -  amount kept for trade
        /// * `order_type` - buy or sell
        pub fn create_order_limit(_origin, base:H256, quote:H256, price:u32,sell_amount:Balance,order_type:OrderType) -> DispatchResult {
            let sender = ensure_signed(_origin)?;

             // call corresponding internal function

             // Return Ok if successful.
             Ok(())
        }

        /// # Provide info to create a trade pair
        /// * `_origin` - signer
        /// * `base` - hash/asset_id of base asset
        /// * `quote` - hash/asset_id of quote asset
        pub fn create_trade_pair(_origin, base:H256, quote:H256) -> DispatchResult {
            let sender = ensure_signed(_origin)?;

             // call corresponding internal function

             // Return Ok if successful.
             Ok(())
        }
    }
}