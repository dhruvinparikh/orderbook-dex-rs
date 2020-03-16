use super::*;

pub trait Trait: assets::Trait + system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Price: Parameter
        + Default
        + Member
        + Bounded
        + AtLeast32Bit
        + Copy
        + From<u128>
        + Into<u128>;
    type PriceFactor: Get<u128>;
    type BlocksPerDay: Get<u32>;
    type OpenedOrdersArrayCap: Get<u8>;
    type ClosedOrdersArrayCap: Get<u8>;
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ExchangePair<T>
where
    T: Trait,
{
    pub hash: T::Hash,
    pub base: T::Hash,
    pub quote: T::Hash,

    pub latest_matched_price: Option<T::Price>,
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

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum OrderStatus {
    Pending,
    PartialFilled, // TODO DP consider partially filled status as well
    Filled,
    Canceled,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
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

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Dex<T>
where
    T: Trait,
{
    pub hash: T::Hash,
    pub base: T::Hash,
    pub quote: T::Hash,
    pub buyer: T::AccountId,      // have base
    pub seller: T::AccountId,     // have quote
    pub maker: T::AccountId,      // create first order
    pub taker: T::AccountId,      // did not create the first order
    pub otype: OrderType,         // taker order's type
    pub price: T::Price,          // maker order's price
    pub base_amount: T::Balance,  // base asset amount to exchange
    pub quote_amount: T::Balance, // quote asset amount to exchange
}

impl<T> LimitOrder<T>
where
    T: Trait,
{
    pub fn new(
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
}

impl<T> Dex<T>
where
    T: Trait,
{
    pub fn new(
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
}

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
        ///	ExchangePairHash => ExchangePair
        pub ExchangePairs get(fn exchange_pair): map hasher(blake2_256) T::Hash => Option<ExchangePair<T>>;
        /// (BaseAssetHash/base_asset_id, quoteAssetHash/quote_asset_id) => ExchangePairHash
        pub ExchangePairsHashByBaseQuote get(fn exchange_pair_hash_by_base_quote): map hasher(blake2_256) (T::Hash, T::Hash) => Option<T::Hash>;
        /// Index => ExchangePairHash
        pub ExchangePairsHashByIndex get(fn exchange_pair_hash_by_index): map hasher(blake2_256) u64 => Option<T::Hash>;
        /// Index
        pub ExchangePairsIndex get(fn exchange_pair_index): u64;
        /// OrderHash => Order
        pub Orders get(fn order): map hasher(blake2_256) T::Hash => Option<LimitOrder<T>>;
        /// (AccoundId, Index) => OrderHash
        pub OwnedOrders get(fn owned_order): map hasher(blake2_256) (T::AccountId, u64) => Option<T::Hash>;
        ///	AccountId => Index
        pub OwnedOrdersIndex get(fn owned_orders_index): map hasher(blake2_256) T::AccountId => u64;
        /// (OrderHash, u64) => DEXHash
        pub OrderOwnedExchanges get(fn order_owned_exchanges): map hasher(blake2_256) (T::Hash, u64) => Option<T::Hash>;
        /// OrderHash => Index
        pub OrderOwnedExchangesIndex get(fn order_owned_exchanges_index): map hasher(blake2_256) T::Hash => u64;
        /// (ExchangePairHash, Index) => OrderHash
        pub ExchangePairOwnedOrders get(fn exchange_pair_owned_order): map hasher(blake2_256) (T::Hash, u64) => Option<T::Hash>;
        /// ExchangePairHash => Index
        pub ExchangePairOwnedOrdersIndex get(fn exchange_pair_owned_order_index): map hasher(blake2_256) T::Hash => u64;

        /// (ExchangePairHash, Price) => LinkedItem
        pub LinkedItemList get(fn linked_item): map hasher(blake2_256) (T::Hash, Option<T::Price>) => Option<OrderLinkedItem<T>>;

        /// DEXHash => DEX
        pub Exchanges get(fn exchange): map hasher(blake2_256) T::Hash => Option<Dex<T>>;

        /// (AccountId, u64) => DEXHash
        pub OwnedExchanges get(fn owned_exchanges): map hasher(blake2_256) (T::AccountId, u64) => Option<T::Hash>;
        /// AccountId => u64
        pub OwnedExchangesIndex get(fn owned_exchanges_index): map hasher(blake2_256) T::AccountId => u64;

        /// (AccountId, ExchangePairHash, u64) => DEXHash
        pub OwnedEPExchanges get(fn owned_ep_exchanges): map hasher(blake2_256) (T::AccountId, T::Hash, u64) => Option<T::Hash>;
        /// (AccountId, ExchangePairHash) => u64
        pub OwnedEPExchangesIndex get(fn owned_ep_exchanges_index): map hasher(blake2_256) (T::AccountId, T::Hash) => u64;

        /// (AccountId, ExchangePairHash) => Vec<OrderHash>
        pub OwnedEPOpenedOrders get(fn owned_ep_opened_orders): map hasher(blake2_256) (T::AccountId, T::Hash) => Option<Vec<T::Hash>>;

        /// (AccountId, ExchangePairHash) => Vec<OrderHash>
        pub OwnedEPClosedOrders get(fn owned_ep_closed_orders): map hasher(blake2_256) (T::AccountId, T::Hash) => Option<Vec<T::Hash>>;

        /// (ExchangePairHash, u64) => DEXHash
        pub ExchangePairOwnedExchanges get(fn exchange_pair_owned_exchanges): map hasher(blake2_256) (T::Hash, u64) => Option<T::Hash>;
        /// ExchangePairHash => u64
        pub ExchangePairOwnedExchangesIndex get(fn exchange_pair_owned_exchanges_index): map hasher(blake2_256) T::Hash => u64;
        /// (ExchangePairHash, BlockNumber) => (Sum_of_Exchange_Volume, Highest_Price, Lowest_Price)
        pub EPExchangeDataBucket get(fn exchange_pair_exchange_data_bucket): map hasher(blake2_256) (T::Hash, T::BlockNumber) => (T::Balance, Option<T::Price>, Option<T::Price>);
        /// store the exchange pair's H/L price within last day
        /// ExchangePairHash => (Vec<Highest_Price>, Vec<Lowest_Price>)
        pub EPExchangePriceBucket get(fn exchange_pair_exchange_price_bucket): map hasher(blake2_256) T::Hash => (Vec<Option<T::Price>>, Vec<Option<T::Price>>);
        pub Nonce: u64;

        pub Orderbook get (fn order_book): Vec<Option<LimitOrder<T>>>;
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

		// (accountId, baseAssetHash/base_asset_id, quoteAssetHash/quote_asset_id, DEXHash/exchange_id, Exchange)
		ExchangeCreated(AccountId, Hash, Hash, Hash, Dex),

		// (accountId, orderHash)
		OrderCanceled(AccountId, Hash),
	}
);

impl<T: Trait> OrderOwnedExchanges<T> {
    pub fn add_exchange(order_hash: T::Hash, exchange_hash: T::Hash) {
        let index = OrderOwnedExchangesIndex::<T>::get(&order_hash);
        Self::insert((order_hash.clone(), index), exchange_hash);
        OrderOwnedExchangesIndex::<T>::insert(order_hash, index + 1);
    }
}

impl<T: Trait> OwnedExchanges<T> {
    pub fn add_exchange(account_id: T::AccountId, exchange_hash: T::Hash) {
        let index = OwnedExchangesIndex::<T>::get(&account_id);
        Self::insert((account_id.clone(), index), exchange_hash);
        OwnedExchangesIndex::<T>::insert(account_id, index + 1);
    }
}

impl<T: Trait> ExchangePairOwnedExchanges<T> {
    pub fn add_exchange(ep_hash: T::Hash, exchange_hash: T::Hash) {
        let index = ExchangePairOwnedExchangesIndex::<T>::get(&ep_hash);
        Self::insert((ep_hash.clone(), index), exchange_hash);
        ExchangePairOwnedExchangesIndex::<T>::insert(ep_hash, index + 1);
    }
}

impl<T: Trait> OwnedEPExchanges<T> {
    pub fn add_exchange(account_id: T::AccountId, ep_hash: T::Hash, exchange_hash: T::Hash) {
        let index = OwnedEPExchangesIndex::<T>::get((account_id.clone(), ep_hash));
        Self::insert((account_id.clone(), ep_hash, index), exchange_hash);
        OwnedEPExchangesIndex::<T>::insert((account_id.clone(), ep_hash), index + 1);
    }
}

impl<T: Trait> OwnedEPOpenedOrders<T> {
    pub fn add_order(account_id: T::AccountId, ep_hash: T::Hash, order_hash: T::Hash) {
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

    pub fn remove_order(account_id: T::AccountId, ep_hash: T::Hash, order_hash: T::Hash) {
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

impl<T: Trait> Orderbook<T> {
    pub fn add_to_order_book(order_hash: T::Hash, limit_order: LimitOrder<T>) {
        // let order = new
        <Orderbook<T>>::mutate(|r| {
            r.push(Some(limit_order));
            (r.len() - 1) as u64;
        });
    }

    pub fn remove_from_order_book(order_hash: T::Hash) {
        // let mut orders;
        // if let Some(ts) = Self::get((account_id.clone(), ep_hash)) {
        //     orders = ts;
        // } else {
        //     orders = Vec::<T::Hash>::new();
        // }

        // orders.retain(|&x| x != order_hash);
        // <OwnedEPOpenedOrders<T>>::insert((account_id, ep_hash), orders);
    }
}

impl<T: Trait> OwnedEPClosedOrders<T> {
    pub fn add_order(account_id: T::AccountId, ep_hash: T::Hash, order_hash: T::Hash) {
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

        pub fn deposit_event() = default;
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
        pub fn cancel_order(origin, _order_hash: T::Hash) -> Result<(), DispatchError> {
            let _sender = ensure_signed(origin)?;

            // call corresponding internal function

             // Return Ok if successful.
             Ok(())
        }
    }
}
