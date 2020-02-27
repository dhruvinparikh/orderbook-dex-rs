use super::*;

impl<T: Trait> Module<T> {
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

    pub fn debug_log_market(ep_hash: T::Hash) {
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
                            let _order = <Orders<T>>::get(order_hash).unwrap();
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
                    let _exchange = <Exchanges<T>>::get(hash).unwrap();
                    // eprintln!("[{}/{}] - {}@{:?}[{:?}]: [Buyer,Seller][{},{}], [Maker,Taker][{},{}], [Base,Quote][{:?}, {:?}]",
                        // exchange.quote, exchange.base, hash, exchange.price, exchange.otype, exchange.buyer, exchange.seller, exchange.maker,
                        // exchange.taker, exchange.base_amount, exchange.quote_amount);
                }
            }

            // eprintln!("[Exchange Pair Data]");
            let _ep = Self::exchange_pair(ep_hash).unwrap();
            // eprintln!("latest matched price: {:?}", ep.latest_matched_price);

            // eprintln!();
        }
    }
}
