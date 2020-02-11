use super::*;

// This function creates token of a given asset
impl<T: Trait> Module<T> {
    pub fn construct(
        _sender: T::AccountId,
        _name: u32,
        asset_id: u32,
        decimals: DNAi64,
        _symbol: u32,
    ) -> DispatchResult {
        // Checking that decimals is non-negative.
        if decimals < DNAi64::from(0) {
            Err("Decimals can't be negative")?
        }

        // Map asset
        if <Self as Store>::Tokens::exists(asset_id) {
            Err("The asset with given id exists.")?
        } else {
            <Self as Store>::OwnerOf::insert(asset_id, _sender);
            <Self as Store>::Tokens::insert(asset_id, (_name, _symbol, decimals));
            <Self as Store>::Decimals::insert(asset_id, decimals);
        }

        // Return Ok.
        Ok(())
    }
}
