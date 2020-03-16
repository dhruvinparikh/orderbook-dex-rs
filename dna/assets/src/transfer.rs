use super::*;

// This function transfers assets from one address to another. If the recipient address doesn't exist, it is created.
impl<T: Trait> Module<T> {
    pub fn transfer(
        sender: T::AccountId,
        hash: T::Hash,
        to: T::AccountId,
        amount: T::Balance,
    ) -> DispatchResult {
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

        let to_amount = Self::balance_of((to.clone(), hash.clone()));
        let new_to_amount = to_amount + amount;
        ensure!(
            new_to_amount <= T::Balance::max_value(),
            Error::<T>::AmountOverflow
        );

        let to_free_amount = Self::free_balance_of((to.clone(), hash.clone()));
        let new_to_free_amount = to_free_amount + amount;
        ensure!(
            new_to_free_amount <= T::Balance::max_value(),
            Error::<T>::AmountOverflow
        );

        BalanceOf::<T>::insert((sender.clone(), hash.clone()), new_from_amount);
        FreeBalanceOf::<T>::insert((sender.clone(), hash.clone()), new_from_free_amount);
        BalanceOf::<T>::insert((to.clone(), hash.clone()), new_to_amount);
        FreeBalanceOf::<T>::insert((to.clone(), hash.clone()), new_to_free_amount);

        Self::deposit_event(RawEvent::Transfered(sender, to, hash, amount));

        // Return Ok.
        Ok(())
    }
}
