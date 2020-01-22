use super::*;

// This function burns tokens of a given asset and from a specific address.
impl<T: Trait> Module<T> {
    pub fn hello(from_address: H256) -> DispatchResult {

        <Self as Store>::TestMap::insert(from_address, from_address);

        // Return Ok.
        Ok(())
    }
}
