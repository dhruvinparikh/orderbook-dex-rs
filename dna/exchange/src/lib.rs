#![cfg_attr(not(feature = "std"), no_std)]
// The above line is needed to compile the Wasm binaries.

// Importing crates declared in the cargo.toml file.
use codec::{Decode, Encode};
use support::{ensure, decl_module, decl_storage, dispatch::Result, StorageMap};
use system::ensure_signed;


// This module's configuration trait.
pub trait Trait: system::Trait {}

#[derive(Encode, Decode)]
pub struct Test {
    pub number: u8,
    pub orderId: u8,
    pub realnumber: u8
}

decl_module! {
pub struct Module<T: Trait> for enum Call where origin: T::Origin
{
    pub fn demand(
        origin,
        number: u8,
        orderId: u8,
        realnumber: u8
    ) -> Result {
        // Ensure we have a signed message, and derive the sender's account id from the signature
        let sender = ensure_signed(origin)?;
       
        Self::create_order(number, orderId, realnumber)
    
}
pub fn offer(
    origin,
    number: u8,
    orderId: u8,
    realnumber: u8
) -> Result {
    // Ensure we have a signed message, and derive the sender's account id from the signature
    let sender = ensure_signed(origin)?;
   
    Self::create_order(number, orderId, realnumber)

}

pub fn matchOffer(
    origin,
    number: u8,
    orderId: u8,
    realnumber: u8
) -> Result {
    // Ensure we have a signed message, and derive the sender's account id from the signature
    let sender = ensure_signed(origin)?;
   
    Self::create_order(number, orderId, realnumber)

}
}
}

impl<T: Trait> Module<T> {
    fn create_order(
        number: u8,
        orderId: u8,
        realnumber: u8
    ) -> Result {
        ensure!(number > 8, "Number argument should be greater than 8");
        ensure!(realnumber > 8, "Number argument should be greater than 8");
        
        Ok(())
    }
}