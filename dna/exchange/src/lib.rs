//! # DEX module
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

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct TradePair<T> where T: Trait {
	hash: T::Hash,
	base: T::Hash,
	quote: T::Hash,
}

#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
	Buy,
	Sell,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum OrderStatus {
	Pending,
	PartialFilled,
	Filled,
	Canceled,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct LimitOrder<T> where T: Trait {
	pub hash: T::Hash,
	pub base: T::Hash,
	pub quote: T::Hash,
	pub owner: T::AccountId,
	pub price: T::Price,
	pub sell_amount: T::Balance,
	pub buy_amount: T::Balance,
	pub remained_sell_amount: T::Balance,
	pub remained_buy_amount: T::Balance,
	pub otype: OrderType,
	pub status: OrderStatus,
}

pub type Price = u128;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Trade<T> where T: Trait {
	hash: T::Hash,
	base: T::Hash,
	quote: T::Hash,
	buyer: T::AccountId, // have base
	seller: T::AccountId, // have quote
	maker: T::AccountId, // create order first
	taker: T::AccountId, // create order not first
	otype: OrderType, // taker order's type
	price: T::Price, // maker order's price
	base_amount: T::Balance, // base token amount to exchange
	quote_amount: T::Balance, // quote token amount to exchange
}

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