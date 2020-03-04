use super::*;

impl<T: Trait> Module<T> {
    pub fn price_as_vec_u8_to_x_by_100m(price: Vec<u8>) -> Result<T::Price, DispatchError> {
        ensure!(price.len() >= 8, Error::<T>::PriceLengthCheckFailed);

        let price = LittleEndian::read_f64(price.as_slice());

        let price_v2 = (T::PriceFactor::get() as f64 * price) as u128;
        let price_v3 = price_v2 as f64 / T::PriceFactor::get() as f64;

        ensure!(price == price_v3, Error::<T>::PriceLengthCheckFailed);

        TryFrom::try_from(price_v2).map_err(|_| Error::<T>::NumberCastError.into())
    }

    pub fn ensure_counterparty_amount_bounds(
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

    pub fn into_128<A: TryInto<u128>>(i: A) -> Result<u128, DispatchError> {
        TryInto::<u128>::try_into(i).map_err(|_| Error::<T>::NumberCastError.into())
    }

    pub fn from_128<A: TryFrom<u128>>(i: u128) -> Result<A, DispatchError> {
        TryFrom::<u128>::try_from(i).map_err(|_| Error::<T>::NumberCastError.into())
    }

    pub fn calculate_ex_amount(
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

            return Ok((Self::from_128(base_qty)?, buyer_order.remained_buy_amount));
        }
    }

    pub fn next_match_price(item: &OrderLinkedItem<T>, otype: OrderType) -> Option<T::Price> {
        if otype == OrderType::Buy {
            item.prev
        } else {
            item.next
        }
    }

    pub fn price_matched(
        order_price: T::Price,
        order_type: OrderType,
        linked_item_price: T::Price,
    ) -> bool {
        match order_type {
            OrderType::Sell => order_price <= linked_item_price,
            OrderType::Buy => order_price >= linked_item_price,
        }
    }
}
