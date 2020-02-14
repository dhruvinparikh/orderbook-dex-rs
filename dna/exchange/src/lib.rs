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
use support::{decl_module, decl_storage,decl_event, dispatch::DispatchResult, StorageMap};
use system::ensure_signed;
use sp_runtime::{traits::{Hash}};

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ExchangePair<T> where T: Trait {
	hash: T::Hash,
	base: T::Hash,
	quote: T::Hash,
}

impl Not for OrderType {
	type Output = OrderType;

	fn not(self) -> Self::Output {
		match self {
			OrderType::Sell => OrderType::Buy,
			OrderType::Buy => OrderType::Sell,
		}
	}
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
	PartialFilled,  // skipping for now
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
	pub otype: OrderType,
	pub status: OrderStatus,
}

pub type Price = u128;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct DEX<T> where T: Trait {
	hash: T::Hash,
	base: T::Hash,
	quote: T::Hash,
	buyer: T::AccountId, // have base
	seller: T::AccountId, // have quote
	maker: T::AccountId, // create first order 
	taker: T::AccountId, // did not create the first order
	otype: OrderType, // taker order's type
	price: T::Price, // maker order's price
	base_amount: T::Balance, // base asset amount to exchange
	quote_amount: T::Balance, // quote asset amount to exchange
}

decl_error! {
	/// Error for the exchange module.
	pub enum Error for Module<T: Trait> {
        // declare error constants here
	}
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as ExchangeStorage {
        ///	ExchangePairHash => DEXPair
		ExchangePairs get(fn dex_pair): map T::Hash => Option<ExchangePair<T>>;

        /// (BaseAssetHash/base_asset_id, quoteAssetHash/quote_asset_id) => ExchangePairHash
        ExchangePairsHashByBaseQuote get(fn exchange_pair_hash_by_base_quote): map (T::Hash, T::Hash) => Option<T::Hash>;
        
		/// OrderHash => Order
		Orders get(fn order): map T::Hash => Option<LimitOrder<T>>;

		/// DEXHash => DEX
		Exchanges get(fn exchange): map T::Hash => Option<Exchange<T>>;

		Nonce: u64;
    }
}

decl_event!(
	pub enum Event<T> 
	where
		<T as system::Trait>::AccountId,
		<T as system::Trait>::Hash,
		ExchangePair = ExchangePair<T>,
		LimitOrder = LimitOrder<T>,
		Exchange = Exchange<T>,
	{
		ExchangePairCreated(AccountId, Hash, ExchangePair),

		// (accountId, baseAssetHash/base_asset_id, quoteAssetHash/quote_asset_id, orderHash/order_id, LimitOrder)
		OrderCreated(AccountId, Hash, Hash, Hash, LimitOrder),

		// (accountId, baseAssetHash/base_asset_id, quoteAssetHash/quote_asset_id, exchangeHash/exchange_id, Exchange)
		ExchangeCreated(AccountId, Hash, Hash, Hash, Exchange),

		// (accountId, orderHash)
		OrderCanceled(AccountId, Hash),
	}
);

// This module's dispatchable functions.
decl_module! {
    // The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event() = default;
        type Error = Error<T>;

        /// # Provide info to create an order limit
        /// * `_origin` - signer
        /// * `base` - hash/asset_id of base asset
        /// * `quote` - hash/asset_id of quote asset
        /// * `price` - price per unit of the base unit
        /// * `sell_amount` -  amount kept for exchange
        /// * `order_type` - buy or sell
        pub fn create_order_limit(_origin, base:H256, quote:H256, price:u32,sell_amount:Balance,order_type:OrderType) -> DispatchResult {
            let sender = ensure_signed(_origin)?;

             // call corresponding internal function

             // Return Ok if successful.
             Ok(())
        }

        /// # Provide info to create a exchange pair
        /// * `_origin` - signer
        /// * `base` - hash/asset_id of base asset
        /// * `quote` - hash/asset_id of quote asset
        pub fn create_exchange_pair(_origin, base:H256, quote:H256) -> DispatchResult {
            let sender = ensure_signed(_origin)?;

             // call corresponding internal function

             // Return Ok if successful.
             Ok(())
        }

        /// # Provide info to cancel an order
        /// * `_origin` - signer
        /// * `order_hash` - hash/order_id of order
        pub fn cancel_order(origin, order_hash: T::Hash) -> Result<(), dispatch::DispatchError> {
			let sender = ensure_signed(origin)?;

			// call corresponding internal function

             // Return Ok if successful.
             Ok(())
		}
    }
}