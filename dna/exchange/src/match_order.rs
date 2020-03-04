use super::*;

impl<T: Trait> Module<T> {
    pub fn order_match(
        ep_hash: T::Hash,
        order: &mut LimitOrder<T>,
    ) -> result::Result<bool, DispatchError> {
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

        if order.status == OrderStatus::Filled {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
