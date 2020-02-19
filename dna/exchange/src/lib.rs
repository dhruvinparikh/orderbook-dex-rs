//! # DEX module
//!

#![cfg_attr(not(feature = "std"), no_std)]
// The above line is needed to compile the Wasm binaries.

// Importing crates declared in the cargo.toml file.
use core::convert::{TryFrom, TryInto};
use primitives::U256;
use rstd::if_std;
use rstd::{ops::Not, prelude::*, result};
use sp_runtime::traits::{Bounded, CheckedSub, Hash, Member, SimpleArithmetic, Zero};
use support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{Get, Randomness},
    Parameter, StorageMap, StorageValue,
};

use byteorder::{ByteOrder, LittleEndian};
use codec::{Decode, Encode};
use system::ensure_signed;

mod types;
mod utils;

#[derive(Encode, Decode, Clone)]
#[cfg_attr(feature = "std", derive(PartialEq, Eq, Debug))]
pub struct LinkedItem<K1, K2, K3> {
    pub prev: Option<K2>,
    pub next: Option<K2>,
    pub price: Option<K2>,
    pub buy_amount: K3,
    pub sell_amount: K3,
    pub orders: Vec<K1>, // TODO DP : remove the item at 0 index will caused performance issue, should be optimized
}

pub trait Trait: assets::Trait + system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Price: Parameter
        + Default
        + Member
        + Bounded
        + SimpleArithmetic
        + Copy
        + From<u128>
        + Into<u128>;
    type PriceFactor: Get<u128>;
    type BlocksPerDay: Get<u32>;
    type OpenedOrdersArrayCap: Get<u8>;
    type ClosedOrdersArrayCap: Get<u8>;
}

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ExchangePair<T>
where
    T: Trait,
{
    hash: T::Hash,
    base: T::Hash,
    quote: T::Hash,

    latest_matched_price: Option<T::Price>,
}

#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Buy,
    Sell,
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

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum OrderStatus {
    Pending,
    PartialFilled, // TODO DP consider partially filled status as well
    Filled,
    Canceled,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct LimitOrder<T>
where
    T: Trait,
{
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
pub struct Dex<T>
where
    T: Trait,
{
    hash: T::Hash,
    base: T::Hash,
    quote: T::Hash,
    buyer: T::AccountId,      // have base
    seller: T::AccountId,     // have quote
    maker: T::AccountId,      // create first order
    taker: T::AccountId,      // did not create the first order
    otype: OrderType,         // taker order's type
    price: T::Price,          // maker order's price
    base_amount: T::Balance,  // base asset amount to exchange
    quote_amount: T::Balance, // quote asset amount to exchange
}

impl<T> LimitOrder<T>
where
    T: Trait,
{
    fn new(
        base: T::Hash,
        quote: T::Hash,
        owner: T::AccountId,
        price: T::Price,
        sell_amount: T::Balance,
        buy_amount: T::Balance,
        otype: OrderType,
    ) -> Self {
        let nonce = Nonce::get();

        let random_seed = <randomness_collective_flip::Module<T>>::random_seed();
        let hash = (
            random_seed,
            <system::Module<T>>::block_number(),
            base,
            quote,
            owner.clone(),
            price,
            sell_amount,
            buy_amount,
            otype,
            nonce,
        )
            .using_encoded(<T as system::Trait>::Hashing::hash);

        LimitOrder {
            hash,
            base,
            quote,
            owner,
            price,
            otype,
            sell_amount,
            buy_amount,
            remained_buy_amount: buy_amount,
            remained_sell_amount: sell_amount,
            status: OrderStatus::Pending,
        }
    }

    pub fn is_finished(&self) -> bool {
        (self.remained_buy_amount == Zero::zero() && self.status == OrderStatus::Filled)
            || self.status == OrderStatus::Canceled
    }

    pub fn debug_log(&self) {
        if_std! {
            // eprintln!("[order]: Base[0x{:02x}], Quote[0x{:02x}], Hash[0x{:02x}], Owner[{:#?}], Price[{:#?}], Type[{:#?}], Status[{:#?}], SellAmount[{:#?}], BuyAmount[{:#?}]", utils::ByteBuf(self.base.as_ref()), utils::ByteBuf(self.quote.as_ref()), utils::ByteBuf(self.hash.as_ref()), self.owner, self.price, self.otype, self.status, self.sell_amount, self.buy_amount);
        }
    }
}

impl<T> Dex<T>
where
    T: Trait,
{
    fn new(
        base: T::Hash,
        quote: T::Hash,
        maker_order: &LimitOrder<T>,
        taker_order: &LimitOrder<T>,
        base_amount: T::Balance,
        quote_amount: T::Balance,
    ) -> Self {
        let nonce = Nonce::get();

        let random_seed = <randomness_collective_flip::Module<T>>::random_seed();
        let hash = (
            random_seed,
            <system::Module<T>>::block_number(),
            nonce,
            maker_order.hash,
            maker_order.remained_sell_amount,
            maker_order.owner.clone(),
            taker_order.hash,
            taker_order.remained_sell_amount,
            taker_order.owner.clone(),
        )
            .using_encoded(<T as system::Trait>::Hashing::hash);

        Nonce::mutate(|x| *x += 1);

        let buyer;
        let seller;
        if taker_order.otype == OrderType::Buy {
            buyer = taker_order.owner.clone();
            seller = maker_order.owner.clone();
        } else {
            buyer = maker_order.owner.clone();
            seller = taker_order.owner.clone();
        }

        Dex {
            hash,
            base,
            quote,
            buyer,
            seller,
            base_amount,
            quote_amount,
            maker: maker_order.owner.clone(),
            taker: taker_order.owner.clone(),
            otype: taker_order.otype,
            price: maker_order.price,
        }
    }

    pub fn debug_log(&self) {
        if_std! {
            // eprintln!("[exchange]: Base[0x{:02x}], Quote[0x{:02x}], Hash[0x{:02x}], buyer[{:#?}], seller[{:#?}], maker[{:#?}], taker[{:#?}], Type[{:#?}], price[{:#?}], base_amout[{:#?}], quote_amout[{:#?}]", utils::ByteBuf(self.base.as_ref()), utils::ByteBuf(self.quote.as_ref()), utils::ByteBuf(self.hash.as_ref()), self.buyer, self.seller, self.maker, self.taker, self.otype, self.price, self.base_amount, self.quote_amount);
        }
    }
}

type OrderLinkedItem<T> =
    types::LinkedItem<<T as system::Trait>::Hash, <T as Trait>::Price, <T as balances::Trait>::Balance>;
type OrderLinkedItemList<T> = types::LinkedList<
    T,
    LinkedItemList<T>,
    <T as system::Trait>::Hash,
    <T as Trait>::Price,
    <T as balances::Trait>::Balance,
>;

decl_error! {
    /// Error for the exchange module.
    pub enum Error for Module<T: Trait> {
        /// Price bounds check failed
        BoundsCheckFailed,
        /// Price length check failed
        PriceLengthCheckFailed,
        /// Number cast error
        NumberCastError,
        /// Overflow error
        OverflowError,
        /// No matching exchange pair
        NoMatchingExchangePair,
        /// Base equals to quote
        BaseEqualQuote,
        /// Asset owner not found
        AssetOwnerNotFound,
        /// Sender not equal to base or quote owner
        SenderNotEqualToBaseOrQuoteOwner,
        /// Same exchange pair with the given base and quote was already exist
        ExchangePairExisted,
        /// Get price error
        OrderMatchGetPriceError,
        /// Get linked list item error
        OrderMatchGetLinkedListItemError,
        /// Get order error
        OrderMatchGetOrderError,
        /// Order match substract error
        OrderMatchSubstractError,
        /// Order match order is not finish
        OrderMatchOrderIsNotFinished,
        /// No matching order
        NoMatchingOrder,
        /// Can only cancel own order
        CanOnlyCancelOwnOrder,
        /// can only cancel not finished order
        CanOnlyCancelNotFinishedOrder,
    }
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as ExchangeStorage {
        ///	ExchangePairHash => DEXPair
        ExchangePairs get(fn exchange_pair): map T::Hash => Option<ExchangePair<T>>;
        /// (BaseAssetHash/base_asset_id, quoteAssetHash/quote_asset_id) => ExchangePairHash
        ExchangePairsHashByBaseQuote get(fn exchange_pair_hash_by_base_quote): map (T::Hash, T::Hash) => Option<T::Hash>;
        /// Index => ExchangePairHash
        ExchangePairsHashByIndex get(fn exchange_pair_hash_by_index): map u64 => Option<T::Hash>;
        /// Index
        ExchangePairsIndex get(fn exchange_pair_index): u64;
        /// OrderHash => Order
        Orders get(fn order): map T::Hash => Option<LimitOrder<T>>;
        /// (AccoundId, Index) => OrderHash
        OwnedOrders get(fn owned_order): map (T::AccountId, u64) => Option<T::Hash>;
        ///	AccountId => Index
        OwnedOrdersIndex get(fn owned_orders_index): map T::AccountId => u64;
        /// (OrderHash, u64) => ExchangeHash
        OrderOwnedExchanges get(fn order_owned_exchanges): map (T::Hash, u64) => Option<T::Hash>;
        /// (OrderHash, u64) => ExchangeHash
        OrderOwnedExchangesIndex get(fn order_owned_exchanges_index): map T::Hash => u64;
        /// (ExchangePairHash, Index) => OrderHash
        ExchangePairOwnedOrders get(fn exchange_pair_owned_order): map (T::Hash, u64) => Option<T::Hash>;
        /// ExchangePairHash => Index
        ExchangePairOwnedOrdersIndex get(fn exchange_pair_owned_order_index): map T::Hash => u64;

        /// (ExchangePairHash, Price) => LinkedItem
        LinkedItemList get(fn linked_item): map (T::Hash, Option<T::Price>) => Option<OrderLinkedItem<T>>;

        /// DEXHash => DEX
        Exchanges get(fn exchange): map T::Hash => Option<Dex<T>>;

        /// (AccountId, u64) => ExchangeHash
        OwnedExchanges get(fn owned_exchanges): map (T::AccountId, u64) => Option<T::Hash>;
        /// AccountId => u64
        OwnedExchangesIndex get(fn owned_exchanges_index): map T::AccountId => u64;

        /// (AccountId, ExchangePairHash, u64) => ExchangeHash
        OwnedEPExchanges get(fn owned_ep_exchanges): map (T::AccountId, T::Hash, u64) => Option<T::Hash>;
        /// (AccountId, ExchangePairHash) => u64
        OwnedEPExchangesIndex get(fn owned_ep_exchanges_index): map (T::AccountId, T::Hash) => u64;

        /// (AccountId, ExchangePairHash) => Vec<OrderHash>
        OwnedEPOpenedOrders get(fn owned_ep_opened_orders): map (T::AccountId, T::Hash) => Option<Vec<T::Hash>>;

        /// (AccountId, ExchangePairHash) => Vec<OrderHash>
        OwnedEPClosedOrders get(fn owned_ep_closed_orders): map (T::AccountId, T::Hash) => Option<Vec<T::Hash>>;

        /// (ExchangePairHash, u64) => ExchangeHash
        ExchangePairOwnedExchanges get(fn exchange_pair_owned_exchanges): map (T::Hash, u64) => Option<T::Hash>;
        /// ExchangePairHash => u64
        ExchangePairOwnedExchangesIndex get(fn exchange_pair_owned_exchanges_index): map T::Hash => u64;
        /// (ExchangePairHash, BlockNumber) => (Sum_of_Exchange_Volume, Highest_Price, Lowest_Price)
        EPExchangeDataBucket get(fn exchange_pair_exchange_data_bucket): map (T::Hash, T::BlockNumber) => (T::Balance, Option<T::Price>, Option<T::Price>);
        /// store the exchange pair's H/L price within last day
        /// ExchangePairHash => (Vec<Highest_Price>, Vec<Lowest_Price>)
        EPExchangePriceBucket get(fn exchange_pair_exchange_price_bucket): map T::Hash => (Vec<Option<T::Price>>, Vec<Option<T::Price>>);
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
		Dex = Dex<T>,
	{
		ExchangePairCreated(AccountId, Hash, ExchangePair),

		// (accountId, baseAssetHash/base_asset_id, quoteAssetHash/quote_asset_id, orderHash/order_id, LimitOrder)
		OrderCreated(AccountId, Hash, Hash, Hash, LimitOrder),

		// (accountId, baseAssetHash/base_asset_id, quoteAssetHash/quote_asset_id, exchangeHash/exchange_id, Exchange)
		ExchangeCreated(AccountId, Hash, Hash, Hash, Dex),

		// (accountId, orderHash)
		OrderCanceled(AccountId, Hash),
	}
);

impl<T: Trait> OrderOwnedExchanges<T> {
    fn add_exchange(order_hash: T::Hash, exchange_hash: T::Hash) {
        let index = OrderOwnedExchangesIndex::<T>::get(&order_hash);
        Self::insert((order_hash.clone(), index), exchange_hash);
        OrderOwnedExchangesIndex::<T>::insert(order_hash, index + 1);
    }
}

impl<T: Trait> OwnedExchanges<T> {
    fn add_exchange(account_id: T::AccountId, exchange_hash: T::Hash) {
        let index = OwnedExchangesIndex::<T>::get(&account_id);
        Self::insert((account_id.clone(), index), exchange_hash);
        OwnedExchangesIndex::<T>::insert(account_id, index + 1);
    }
}

impl<T: Trait> ExchangePairOwnedExchanges<T> {
    fn add_exchange(ep_hash: T::Hash, exchange_hash: T::Hash) {
        let index = ExchangePairOwnedExchangesIndex::<T>::get(&ep_hash);
        Self::insert((ep_hash.clone(), index), exchange_hash);
        ExchangePairOwnedExchangesIndex::<T>::insert(ep_hash, index + 1);
    }
}

impl<T: Trait> OwnedEPExchanges<T> {
    fn add_exchange(account_id: T::AccountId, ep_hash: T::Hash, exchange_hash: T::Hash) {
        let index = OwnedEPExchangesIndex::<T>::get((account_id.clone(), ep_hash));
        Self::insert((account_id.clone(), ep_hash, index), exchange_hash);
        OwnedEPExchangesIndex::<T>::insert((account_id.clone(), ep_hash), index + 1);
    }
}

impl<T: Trait> OwnedEPOpenedOrders<T> {
    fn add_order(account_id: T::AccountId, ep_hash: T::Hash, order_hash: T::Hash) {
        let mut orders;
        if let Some(ts) = Self::get((account_id.clone(), ep_hash)) {
            orders = ts;
        } else {
            orders = Vec::<T::Hash>::new();
        }

        match orders.iter().position(|&x| x == order_hash) {
            Some(_) => return,
            None => {
                orders.insert(0, order_hash);
                if orders.len() == T::OpenedOrdersArrayCap::get() as usize {
                    orders.pop();
                }

                <OwnedEPOpenedOrders<T>>::insert((account_id, ep_hash), orders);
            }
        }
    }

    fn remove_order(account_id: T::AccountId, ep_hash: T::Hash, order_hash: T::Hash) {
        let mut orders;
        if let Some(ts) = Self::get((account_id.clone(), ep_hash)) {
            orders = ts;
        } else {
            orders = Vec::<T::Hash>::new();
        }

        orders.retain(|&x| x != order_hash);
        <OwnedEPOpenedOrders<T>>::insert((account_id, ep_hash), orders);
    }
}

impl<T: Trait> OwnedEPClosedOrders<T> {
    fn add_order(account_id: T::AccountId, ep_hash: T::Hash, order_hash: T::Hash) {
        let mut orders;
        if let Some(ts) = Self::get((account_id.clone(), ep_hash)) {
            orders = ts;
        } else {
            orders = Vec::<T::Hash>::new();
        }

        match orders.iter().position(|&x| x == order_hash) {
            Some(_) => return,
            None => {
                orders.insert(0, order_hash);
                if orders.len() == T::ClosedOrdersArrayCap::get() as usize {
                    orders.pop();
                }

                <OwnedEPClosedOrders<T>>::insert((account_id, ep_hash), orders);
            }
        }
    }
}

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
        pub fn create_order(_origin, base: T::Hash, quote: T::Hash, otype: OrderType, price: T::Price, sell_amount: T::Balance) -> DispatchResult {
            let sender = ensure_signed(_origin)?;

             // call corresponding internal function
             Self::do_create_limit_order(sender, base, quote, otype, price, sell_amount)?;

             // Return Ok if successful.
             Ok(())
        }

        /// # Provide info to create an order limit
        /// * `_origin` - signer
        /// * `base` - hash/asset_id of base asset
        /// * `quote` - hash/asset_id of quote asset
        /// * `price` - price per unit of the base unit
        /// * `sell_amount` -  amount kept for exchange
        /// * `order_type` - buy or sell
        pub fn create_order_with_decimals(_origin, base: T::Hash, quote: T::Hash, otype: OrderType, price: Vec<u8>, sell_amount: T::Balance) -> DispatchResult {
            let sender = ensure_signed(_origin)?;
            let price = Self::price_as_vec_u8_to_x_by_100m(price)?;

             // call corresponding internal function
             Self::do_create_limit_order(sender, base, quote, otype, price, sell_amount)?;

             // Return Ok if successful.
             Ok(())
        }

        /// # Provide info to create a exchange pair
        /// * `_origin` - signer
        /// * `base` - hash/asset_id of base asset
        /// * `quote` - hash/asset_id of quote asset
        pub fn create_exchange_pair(_origin, base:T::Hash, quote:T::Hash) -> DispatchResult {
            let sender = ensure_signed(_origin)?;

             // call corresponding internal function
            Self::do_create_exchange_pair(sender, base, quote)?;

             // Return Ok if successful.
             Ok(())
        }

        /// # Provide info to cancel an order
        /// * `_origin` - signer
        /// * `order_hash` - hash/order_id of order
        pub fn cancel_order(origin, order_hash: T::Hash) -> Result<(), DispatchError> {
            let sender = ensure_signed(origin)?;

            // call corresponding internal function

             // Return Ok if successful.
             Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn ensure_bounds(price: T::Price, sell_amount: T::Balance) -> DispatchResult {
        ensure!(
            price > Zero::zero() && price <= T::Price::max_value(),
            Error::<T>::BoundsCheckFailed
        );
        ensure!(
            sell_amount > Zero::zero() && sell_amount <= T::Balance::max_value(),
            Error::<T>::BoundsCheckFailed
        );
        Ok(())
    }

    fn price_as_vec_u8_to_x_by_100m(price: Vec<u8>) -> Result<T::Price, DispatchError> {
        ensure!(price.len() >= 8, Error::<T>::PriceLengthCheckFailed);

        let price = LittleEndian::read_f64(price.as_slice());

        let price_v2 = (T::PriceFactor::get() as f64 * price) as u128;
        let price_v3 = price_v2 as f64 / T::PriceFactor::get() as f64;

        ensure!(price == price_v3, Error::<T>::PriceLengthCheckFailed);

        TryFrom::try_from(price_v2).map_err(|_| Error::<T>::NumberCastError.into())
    }

    fn ensure_counterparty_amount_bounds(
        otype: OrderType,
        price: T::Price,
        amount: T::Balance,
    ) -> result::Result<T::Balance, DispatchError> {
        let price_u256 = U256::from(Self::into_128(price)?);
        let amount_u256 = U256::from(Self::into_128(amount)?);
        let max_balance_u256 = U256::from(Self::into_128(T::Balance::max_value())?);
        let price_factor_u256 = U256::from(T::PriceFactor::get());

        let amount_v2: U256;
        let counterparty_amount: U256;

        match otype {
            OrderType::Buy => {
                counterparty_amount = amount_u256 * price_factor_u256 / price_u256;
                amount_v2 = counterparty_amount * price_u256 / price_factor_u256;
            }
            OrderType::Sell => {
                counterparty_amount = amount_u256 * price_u256 / price_factor_u256;
                amount_v2 = counterparty_amount * price_factor_u256 / price_u256;
            }
        }

        ensure!(amount_u256 == amount_v2, Error::<T>::BoundsCheckFailed);
        ensure!(
            counterparty_amount != 0.into() && counterparty_amount <= max_balance_u256,
            Error::<T>::BoundsCheckFailed
        );

        // todo: change to u128
        let result: u128 = counterparty_amount
            .try_into()
            .map_err(|_| Error::<T>::OverflowError)?;

        Self::from_128(result)
    }

    fn ensure_exchange_pair(
        base: T::Hash,
        quote: T::Hash,
    ) -> result::Result<T::Hash, DispatchError> {
        let bq = Self::exchange_pair_hash_by_base_quote((base, quote));
        ensure!(bq.is_some(), Error::<T>::NoMatchingExchangePair);

        match bq {
            Some(bq) => Ok(bq),
            None => Err(Error::<T>::NoMatchingExchangePair.into()),
        }
    }

    fn do_create_exchange_pair(
        sender: T::AccountId,
        base: T::Hash,
        quote: T::Hash,
    ) -> DispatchResult {
        ensure!(base != quote, Error::<T>::BaseEqualQuote);

        let base_owner = <assets::Module<T>>::owner(base);
        let quote_owner = <assets::Module<T>>::owner(quote);

        ensure!(
            base_owner.is_some() && quote_owner.is_some(),
            Error::<T>::AssetOwnerNotFound
        );

        let base_owner = base_owner.unwrap();
        let quote_owner = quote_owner.unwrap();

        ensure!(
            sender == base_owner || sender == quote_owner,
            Error::<T>::SenderNotEqualToBaseOrQuoteOwner
        );

        let bq = Self::exchange_pair_hash_by_base_quote((base, quote));
        let qb = Self::exchange_pair_hash_by_base_quote((quote, base));

        ensure!(
            !bq.is_some() && !qb.is_some(),
            Error::<T>::ExchangePairExisted
        );

        let nonce = Nonce::get();

        let random_seed = <randomness_collective_flip::Module<T>>::random_seed();
        let hash = (
            random_seed,
            <system::Module<T>>::block_number(),
            sender.clone(),
            base,
            quote,
            nonce,
        )
            .using_encoded(<T as system::Trait>::Hashing::hash);

        let ep = ExchangePair {
            hash,
            base,
            quote,
            latest_matched_price: None,
        };

        Nonce::mutate(|n| *n += 1);
        ExchangePairs::insert(hash, ep.clone());
        ExchangePairsHashByBaseQuote::<T>::insert((base, quote), hash);

        let index = Self::exchange_pair_index();
        ExchangePairsHashByIndex::<T>::insert(index, hash);
        ExchangePairsIndex::mutate(|n| *n += 1);

        Self::deposit_event(RawEvent::ExchangePairCreated(sender, hash, ep));

        Ok(())
    }

    fn do_create_limit_order(
        sender: T::AccountId,
        base: T::Hash,
        quote: T::Hash,
        otype: OrderType,
        price: T::Price,
        sell_amount: T::Balance,
    ) -> DispatchResult {
        if_std! {
            // eprintln!("create limit order begin");
        }

        Self::ensure_bounds(price, sell_amount)?;
        let buy_amount = Self::ensure_counterparty_amount_bounds(otype, price, sell_amount)?;

        let ep_hash = Self::ensure_exchange_pair(base, quote)?;

        let op_asset_hash;
        match otype {
            OrderType::Buy => op_asset_hash = base,
            OrderType::Sell => op_asset_hash = quote,
        };

        let mut order = LimitOrder::new(
            base,
            quote,
            sender.clone(),
            price,
            sell_amount,
            buy_amount,
            otype,
        );
        let hash = order.hash;

        <assets::Module<T>>::ensure_free_balance(sender.clone(), op_asset_hash, sell_amount)?;
        <assets::Module<T>>::freeze(sender.clone(), op_asset_hash, sell_amount)?;
        Orders::insert(hash, order.clone());
        Nonce::mutate(|n| *n += 1);
        Self::deposit_event(RawEvent::OrderCreated(
            sender.clone(),
            base,
            quote,
            hash,
            order.clone(),
        ));
        <OwnedEPOpenedOrders<T>>::add_order(sender.clone(), ep_hash, order.hash);

        order.debug_log();

        let owned_index = Self::owned_orders_index(sender.clone());
        OwnedOrders::<T>::insert((sender.clone(), owned_index), hash);
        OwnedOrdersIndex::<T>::insert(sender.clone(), owned_index + 1);

        let ep_owned_index = Self::exchange_pair_owned_order_index(ep_hash);
        ExchangePairOwnedOrders::<T>::insert((ep_hash, ep_owned_index), hash);
        ExchangePairOwnedOrdersIndex::<T>::insert(ep_hash, ep_owned_index + 1);

        // order match
        let filled = Self::order_match(ep_hash, &mut order)?;

        // add order to the market order list
        if !filled {
            <OrderLinkedItemList<T>>::append(
                ep_hash,
                price,
                hash,
                order.remained_sell_amount,
                order.remained_buy_amount,
                otype,
            );
            Self::debug_log_market(ep_hash);
        } else {
            <OwnedEPOpenedOrders<T>>::remove_order(sender.clone(), ep_hash, order.hash);
            <OwnedEPClosedOrders<T>>::add_order(sender.clone(), ep_hash, order.hash);
        }

        if_std! {
            // eprintln!("create limit order end");
        }

        Ok(())
    }

    fn order_match(
        ep_hash: T::Hash,
        order: &mut LimitOrder<T>,
    ) -> result::Result<bool, DispatchError> {
        if_std! {
            // eprintln!("order match begin");
        }

        let mut head = <OrderLinkedItemList<T>>::read_head(ep_hash);

        let end_item_price;
        let otype = order.otype;
        let oprice = order.price;

        if otype == OrderType::Buy {
            end_item_price = Some(T::Price::min_value());
        } else {
            end_item_price = Some(T::Price::max_value());
        }

        let ep = Self::exchange_pair(ep_hash).ok_or(Error::<T>::NoMatchingExchangePair)?;
        let give: T::Hash;
        let have: T::Hash;

        match otype {
            OrderType::Buy => {
                give = ep.base;
                have = ep.quote;
            }
            OrderType::Sell => {
                give = ep.quote;
                have = ep.base;
            }
        };

        loop {
            if order.status == OrderStatus::Filled {
                break;
            }

            let item_price = Self::next_match_price(&head, !otype);

            if item_price == end_item_price {
                break;
            }

            let item_price = item_price.ok_or(Error::<T>::OrderMatchGetPriceError)?;

            if !Self::price_matched(oprice, otype, item_price) {
                break;
            }

            let item = <LinkedItemList<T>>::get((ep_hash, Some(item_price)))
                .ok_or(Error::<T>::OrderMatchGetLinkedListItemError)?;
            for o in item.orders.iter() {
                let mut o = Self::order(o).ok_or(Error::<T>::OrderMatchGetOrderError)?;

                let (base_qty, quote_qty) = Self::calculate_ex_amount(&o, &order)?;

                let give_qty: T::Balance;
                let have_qty: T::Balance;
                match otype {
                    OrderType::Buy => {
                        give_qty = base_qty;
                        have_qty = quote_qty;
                    }
                    OrderType::Sell => {
                        give_qty = quote_qty;
                        have_qty = base_qty;
                    }
                };

                if order.remained_sell_amount == order.sell_amount {
                    order.status = OrderStatus::PartialFilled;
                }

                if o.remained_sell_amount == o.sell_amount {
                    o.status = OrderStatus::PartialFilled;
                }

                <assets::Module<T>>::unfreeze(order.owner.clone(), give, give_qty)?;
                <assets::Module<T>>::unfreeze(o.owner.clone(), have, have_qty)?;

                <assets::Module<T>>::transfer(
                    order.owner.clone(),
                    give,
                    o.owner.clone(),
                    give_qty,
                )?;
                <assets::Module<T>>::transfer(
                    o.owner.clone(),
                    have,
                    order.owner.clone(),
                    have_qty,
                )?;

                order.remained_sell_amount = order
                    .remained_sell_amount
                    .checked_sub(&give_qty)
                    .ok_or(Error::<T>::OrderMatchSubstractError)?;
                order.remained_buy_amount = order
                    .remained_buy_amount
                    .checked_sub(&have_qty)
                    .ok_or(Error::<T>::OrderMatchSubstractError)?;

                o.remained_sell_amount = o
                    .remained_sell_amount
                    .checked_sub(&have_qty)
                    .ok_or(Error::<T>::OrderMatchSubstractError)?;
                o.remained_buy_amount = o
                    .remained_buy_amount
                    .checked_sub(&give_qty)
                    .ok_or(Error::<T>::OrderMatchSubstractError)?;

                if order.remained_buy_amount == Zero::zero() {
                    order.status = OrderStatus::Filled;
                    if order.remained_sell_amount != Zero::zero() {
                        <assets::Module<T>>::unfreeze(
                            order.owner.clone(),
                            give,
                            order.remained_sell_amount,
                        )?;
                        order.remained_sell_amount = Zero::zero();
                    }

                    <OwnedEPOpenedOrders<T>>::remove_order(
                        order.owner.clone(),
                        ep_hash,
                        order.hash,
                    );
                    <OwnedEPClosedOrders<T>>::add_order(order.owner.clone(), ep_hash, order.hash);

                    ensure!(
                        order.is_finished(),
                        Error::<T>::OrderMatchOrderIsNotFinished
                    );
                }

                if o.remained_buy_amount == Zero::zero() {
                    o.status = OrderStatus::Filled;
                    if o.remained_sell_amount != Zero::zero() {
                        <assets::Module<T>>::unfreeze(
                            o.owner.clone(),
                            have,
                            o.remained_sell_amount,
                        )?;
                        o.remained_sell_amount = Zero::zero();
                    }

                    <OwnedEPOpenedOrders<T>>::remove_order(o.owner.clone(), ep_hash, o.hash);
                    <OwnedEPClosedOrders<T>>::add_order(o.owner.clone(), ep_hash, o.hash);

                    ensure!(o.is_finished(), Error::<T>::OrderMatchOrderIsNotFinished);
                }

                Orders::insert(order.hash.clone(), order.clone());
                Orders::insert(o.hash.clone(), o.clone());

                // save the exchange pair market data
                Self::set_ep_market_data(ep_hash, o.price, quote_qty)?;

                // update maker order's amount in market
                <OrderLinkedItemList<T>>::update_amount(ep_hash, o.price, have_qty, give_qty);

                // remove the matched order
                <OrderLinkedItemList<T>>::remove_all(ep_hash, !otype);

                // save the exchange data
                let dex = Dex::new(ep.base, ep.quote, &o, &order, base_qty, quote_qty);
                Exchanges::insert(dex.hash, dex.clone());

                Self::deposit_event(RawEvent::ExchangeCreated(
                    order.owner.clone(),
                    ep.base,
                    ep.quote,
                    dex.hash,
                    dex.clone(),
                ));

                order.debug_log();
                o.debug_log();
                dex.debug_log();
                Self::debug_log_market(ep_hash);

                // save exchange reference data to store
                <OrderOwnedExchanges<T>>::add_exchange(order.hash, dex.hash);
                <OrderOwnedExchanges<T>>::add_exchange(o.hash, dex.hash);

                <OwnedExchanges<T>>::add_exchange(order.owner.clone(), dex.hash);
                <OwnedExchanges<T>>::add_exchange(o.owner.clone(), dex.hash);

                <OwnedEPExchanges<T>>::add_exchange(order.owner.clone(), ep_hash, dex.hash);
                <OwnedEPExchanges<T>>::add_exchange(o.owner.clone(), ep_hash, dex.hash);
                <ExchangePairOwnedExchanges<T>>::add_exchange(ep_hash, dex.hash);

                if order.status == OrderStatus::Filled {
                    break;
                }
            }

            head = <OrderLinkedItemList<T>>::read_head(ep_hash);
        }

        if_std! {
            // eprintln!("order match end");
        }

        if order.status == OrderStatus::Filled {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn into_128<A: TryInto<u128>>(i: A) -> Result<u128, DispatchError> {
        TryInto::<u128>::try_into(i).map_err(|_| Error::<T>::NumberCastError.into())
    }

    fn from_128<A: TryFrom<u128>>(i: u128) -> Result<A, DispatchError> {
        TryFrom::<u128>::try_from(i).map_err(|_| Error::<T>::NumberCastError.into())
    }

    fn calculate_ex_amount(
        maker_order: &LimitOrder<T>,
        taker_order: &LimitOrder<T>,
    ) -> result::Result<(T::Balance, T::Balance), DispatchError> {
        let buyer_order;
        let seller_order;
        if taker_order.otype == OrderType::Buy {
            buyer_order = taker_order;
            seller_order = maker_order;
        } else {
            buyer_order = maker_order;
            seller_order = taker_order;
        }

        // TODO DP: overflow checked need
        let mut seller_order_filled = true;
        if seller_order.remained_buy_amount <= buyer_order.remained_sell_amount {
            // seller_order is Filled
            let quote_qty: u128 = Self::into_128(seller_order.remained_buy_amount)?
                * T::PriceFactor::get()
                / maker_order.price.into();
            if Self::into_128(buyer_order.remained_buy_amount)? < quote_qty {
                seller_order_filled = false;
            }
        } else {
            let base_qty: u128 = Self::into_128(buyer_order.remained_buy_amount)?
                * maker_order.price.into()
                / T::PriceFactor::get();
            if Self::into_128(seller_order.remained_buy_amount)? >= base_qty {
                seller_order_filled = false;
            }
        }

        // if seller_order.remained_buy_amount <= buyer_order.remained_sell_amount { // seller_order is Filled
        if seller_order_filled {
            let mut quote_qty: u128 = Self::into_128(seller_order.remained_buy_amount)?
                * T::PriceFactor::get()
                / maker_order.price.into();
            let buy_amount_v2 =
                quote_qty * Self::into_128(maker_order.price)? / T::PriceFactor::get();
            if buy_amount_v2 != Self::into_128(seller_order.remained_buy_amount)?
                && Self::into_128(buyer_order.remained_buy_amount)? > quote_qty
            // have fraction, seller(Filled) give more to align
            {
                quote_qty = quote_qty + 1;
            }

            if_std! {
                // eprintln!("1, calculate exchange amount");
                // eprintln!("match price is: {:#?}", maker_order.price);
                // eprintln!("seller order give amount (seller_order.remain_buy_amount): {:#?}", seller_order.remained_buy_amount);
                // eprintln!("buy order give amount: {:#?}", quote_qty);
                // eprintln!("maker order: ");
                maker_order.debug_log();
                // eprintln!("taker order: ");
                taker_order.debug_log();
            }

            return Ok((seller_order.remained_buy_amount, Self::from_128(quote_qty)?));
        } else {
            // buyer_order is Filled
            let mut base_qty: u128 = Self::into_128(buyer_order.remained_buy_amount)?
                * maker_order.price.into()
                / T::PriceFactor::get();
            let buy_amount_v2 = base_qty * T::PriceFactor::get() / maker_order.price.into();
            if buy_amount_v2 != Self::into_128(buyer_order.remained_buy_amount)?
                && Self::into_128(seller_order.remained_buy_amount)? > base_qty
            // have fraction, buyer(Filled) give more to align
            {
                base_qty = base_qty + 1;
            }

            if_std! {
                // eprintln!("2, calculate exchange amount");
                // eprintln!("match price is: {:#?}", maker_order.price);
                // eprintln!("seller order give amount : {:#?}", base_qty);
                // eprintln!("buy order give amount (buyer_order.remain_buy_amount): {:#?}", buyer_order.remained_buy_amount);
                // eprintln!("maker order: ");
                maker_order.debug_log();
                // eprintln!("taker order: ");
                taker_order.debug_log();
            }

            return Ok((Self::from_128(base_qty)?, buyer_order.remained_buy_amount));
        }
    }

    fn next_match_price(item: &OrderLinkedItem<T>, otype: OrderType) -> Option<T::Price> {
        if otype == OrderType::Buy {
            item.prev
        } else {
            item.next
        }
    }

    fn price_matched(
        order_price: T::Price,
        order_type: OrderType,
        linked_item_price: T::Price,
    ) -> bool {
        match order_type {
            OrderType::Sell => order_price <= linked_item_price,
            OrderType::Buy => order_price >= linked_item_price,
        }
    }

    pub fn set_ep_market_data(
        ep_hash: T::Hash,
        price: T::Price,
        amount: T::Balance,
    ) -> DispatchResult {
        let mut ep = <ExchangePairs<T>>::get(ep_hash).ok_or(Error::<T>::NoMatchingExchangePair)?;

        ep.latest_matched_price = Some(price);

        let mut bucket =
            <EPExchangeDataBucket<T>>::get((ep_hash, <system::Module<T>>::block_number()));
        bucket.0 = bucket.0 + amount;

        match bucket.1 {
            Some(ep_h_price) => {
                if price > ep_h_price {
                    bucket.1 = Some(price);
                }
            }
            None => {
                bucket.1 = Some(price);
            }
        }

        match bucket.2 {
            Some(ep_l_price) => {
                if price < ep_l_price {
                    bucket.2 = Some(price);
                }
            }
            None => {
                bucket.2 = Some(price);
            }
        }

        <EPExchangeDataBucket<T>>::insert((ep_hash, <system::Module<T>>::block_number()), bucket);
        <ExchangePairs<T>>::insert(ep_hash, ep);

        Ok(())
    }

    fn debug_log_market(ep_hash: T::Hash) {
        if_std! {
            let mut item = <OrderLinkedItemList<T>>::read_bottom(ep_hash);

            // eprintln!("[Market Orders]");

            loop {
                if item.price == Some(T::Price::min_value()) {
                    // eprint!("Bottom ==> ");
                } else if item.price == Some(T::Price::max_value()) {
                    // eprint!("Top ==> ");
                } else if item.price == None {
                    // eprint!("Head ==> ");
                }

                // eprint!("Price({:?}), Next({:?}), Prev({:?}), Sell_Amount({:?}), Buy_Amount({:?}), Orders({}): ",
                    // item.price, item.next, item.prev, item.sell_amount, item.buy_amount, item.orders.len());

                let mut orders = item.orders.iter();
                loop {
                    match orders.next() {
                        Some(order_hash) => {
                            let order = <Orders<T>>::get(order_hash).unwrap();
                            // eprint!("({}@[{:?}]: Sell[{:?}, {:?}], Buy[{:?}, {:?}]), ", order.hash, order.status,
                                // order.sell_amount, order.remained_sell_amount, order.buy_amount, order.remained_buy_amount);
                        },
                        None => break,
                    }
                }

                // eprintln!("");

                if item.next == Some(T::Price::min_value()) {
                    break;
                } else {
                    item = OrderLinkedItemList::<T>::read(ep_hash, item.next);
                }
            }

            // eprintln!("[Market Exchanges]");

            let index_end = Self::exchange_pair_owned_exchanges_index(ep_hash);
            for i in 0..index_end {
                let hash = Self::exchange_pair_owned_exchanges((ep_hash, i));
                if let Some(hash) = hash {
                    let exchange = <Exchanges<T>>::get(hash).unwrap();
                    // eprintln!("[{}/{}] - {}@{:?}[{:?}]: [Buyer,Seller][{},{}], [Maker,Taker][{},{}], [Base,Quote][{:?}, {:?}]",
                        // exchange.quote, exchange.base, hash, exchange.price, exchange.otype, exchange.buyer, exchange.seller, exchange.maker,
                        // exchange.taker, exchange.base_amount, exchange.quote_amount);
                }
            }

            // eprintln!("[Exchange Pair Data]");
            let ep = Self::exchange_pair(ep_hash).unwrap();
            // eprintln!("latest matched price: {:?}", ep.latest_matched_price);

            // eprintln!();
        }
    }
}
