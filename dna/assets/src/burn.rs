use super::*;

// This function burns assets from a specific address.
impl<T: Trait> Module<T> {
    pub fn burn(sender: T::AccountId, hash: T::Hash, amount: T::Balance) -> DispatchResult {
        // Checking that amount is non-negative.
        ensure!(amount >= T::Balance::from(0), Error::<T>::NegativeAmount);

        let asset = Self::asset(hash);
        ensure!(asset.is_some(), Error::<T>::NoMatchingAsset);

        ensure!(
            <FreeBalanceOf<T>>::contains_key((sender.clone(), hash)),
            Error::<T>::SenderHaveNoAsset
        );

        let from_amount = Self::balance_of((sender.clone(), hash.clone()));
        ensure!(from_amount >= amount, Error::<T>::BalanceNotEnough);
        let new_from_amount = from_amount - amount;

        let from_free_amount = Self::free_balance_of((sender.clone(), hash.clone()));
        ensure!(from_free_amount >= amount, Error::<T>::BalanceNotEnough);
        let new_from_free_amount = from_free_amount - amount;

        BalanceOf::<T>::insert((sender.clone(), hash.clone()), new_from_amount);
        FreeBalanceOf::<T>::insert((sender.clone(), hash.clone()), new_from_free_amount);

        Self::deposit_event(RawEvent::Burned(sender, hash, amount));

        // Return Ok.
        Ok(())
    }
}
