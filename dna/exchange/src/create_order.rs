use super::*;

impl<T: Trait> Module<T> {
    pub fn do_create_limit_order(
        sender: T::AccountId,
        base: T::Hash,
        quote: T::Hash,
        otype: OrderType,
        price: T::Price,
        sell_amount: T::Balance,
    ) -> DispatchResult {
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
        <Orderbook<T>>::add_to_order_book(order.hash, order.clone());
        Self::deposit_event(RawEvent::OrderCreated(
            sender.clone(),
            base,
            quote,
            hash,
            order.clone(),
        ));
        <OwnedEPOpenedOrders<T>>::add_order(sender.clone(), ep_hash, order.hash);

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
        } else {
            <OwnedEPOpenedOrders<T>>::remove_order(sender.clone(), ep_hash, order.hash);
            <OwnedEPClosedOrders<T>>::add_order(sender.clone(), ep_hash, order.hash);
        }
        // let i = <OrderBook<T>>::mutate(|r| {
        //     r.push(Some(order));
        //     owned_index;
        // });
        // Self::deposit_event(RawEvent::RegistrarAdded(i));

        Ok(())
    }

    pub fn ensure_bounds(price: T::Price, sell_amount: T::Balance) -> DispatchResult {
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
}
