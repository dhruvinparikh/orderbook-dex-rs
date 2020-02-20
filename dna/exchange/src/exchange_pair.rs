use super::*;

impl<T: Trait> Module<T> {
    pub fn do_create_exchange_pair(
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

    pub fn ensure_exchange_pair(
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
}
