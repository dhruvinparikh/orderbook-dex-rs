use super::*;

// This function creates assets and deposits them into an address. If the recipient address doesn't exist, it is created.
impl<T: Trait> Module<T> {
    pub fn mint(origin: T::Origin, symbol: Vec<u8>, total_supply: T::Balance) -> DispatchResult {
        // // Checking that total_supply is non-negative.
        ensure!(
            total_supply >= T::Balance::from(0),
            Error::<T>::NegativeAmount
        );
        let sender = ensure_signed(origin)?;

        let nonce = Nonce::get();

        let random_seed = <randomness_collective_flip::Module<T>>::random_seed();
        let hash =
            (random_seed, sender.clone(), nonce).using_encoded(<T as system::Trait>::Hashing::hash);

        let asset = Asset::<T::Hash, T::Balance> {
            hash: hash.clone(),
            total_supply,
            symbol: symbol.clone(),
        };

        Nonce::mutate(|n| *n += 1);
        <Assets<T>>::insert(hash.clone(), asset);
        Owners::<T>::insert(hash.clone(), sender.clone());
        BalanceOf::<T>::insert((sender.clone(), hash.clone()), total_supply);
        FreeBalanceOf::<T>::insert((sender.clone(), hash.clone()), total_supply);

        let owned_asset_index = OwnedAssetsIndex::<T>::get(sender.clone());
        OwnedAssets::<T>::insert((sender.clone(), owned_asset_index), hash);
        OwnedAssetsIndex::<T>::insert(sender.clone(), owned_asset_index + 1);

        Self::deposit_event(RawEvent::Issued(sender, hash.clone(), total_supply));

        // Return Ok.
        Ok(())
    }
}
