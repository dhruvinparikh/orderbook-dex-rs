use super::*;

// This function unfreezes assets of a specific address.
impl<T: Trait> Module<T> {
    pub fn unfreeze(sender: T::AccountId, hash: T::Hash, amount: T::Balance) -> DispatchResult {
        let asset = Self::asset(hash);
        ensure!(asset.is_some(), Error::<T>::NoMatchingAsset);

        ensure!(
            FreeBalanceOf::<T>::exists((sender.clone(), hash)),
            Error::<T>::SenderHaveNoAsset
        );

        let old_freezed_amount = Self::freezed_balance_of((sender.clone(), hash.clone()));
        ensure!(old_freezed_amount >= amount, Error::<T>::BalanceNotEnough);

        let old_free_amount = Self::free_balance_of((sender.clone(), hash.clone()));
        ensure!(
            old_free_amount + amount <= T::Balance::max_value(),
            Error::<T>::AmountOverflow
        );

        FreeBalanceOf::<T>::insert((sender.clone(), hash.clone()), old_free_amount + amount);
        FreezedBalanceOf::<T>::insert((sender.clone(), hash.clone()), old_freezed_amount - amount);

        Self::deposit_event(RawEvent::UnFreezed(sender, hash, amount));

        Ok(())
    }
    pub fn ensure_free_balance(
        sender: T::AccountId,
        hash: T::Hash,
        amount: T::Balance,
    ) -> DispatchResult {
        let asset = Self::asset(hash);
        ensure!(asset.is_some(), Error::<T>::NoMatchingAsset);

        ensure!(
            FreeBalanceOf::<T>::exists((sender.clone(), hash.clone())),
            Error::<T>::SenderHaveNoAsset
        );

        let free_amount = Self::free_balance_of((sender.clone(), hash.clone()));
        ensure!(free_amount >= amount, Error::<T>::BalanceNotEnough);

        Ok(())
    }
}
