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
}
