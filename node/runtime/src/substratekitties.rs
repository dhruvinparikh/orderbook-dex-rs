#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use runtime_io::hashing::blake2_128;
use sr_primitives::traits::{Bounded, Member, One, SimpleArithmetic};
use support::traits::{Currency, Randomness};
/// A runtime module for managing non-fungible tokens
use support::{decl_event, decl_module, decl_storage, ensure, Parameter};
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type KittyIndex: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
    type Currency: Currency<Self::AccountId>;
    type Randomness: Randomness<Self::Hash>;
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Kitty(pub [u8; 16]);

decl_event!(
	pub enum Event<T> where
		<T as system::Trait>::AccountId,
		<T as Trait>::KittyIndex,
		Balance = BalanceOf<T>,
	{
		/// A kitty is created. (owner, kitty_id)
		Created(AccountId, KittyIndex),
		/// A kitty is transferred. (from, to, kitty_id)
		Transferred(AccountId, AccountId, KittyIndex),
		/// A kitty is available for sale. (owner, kitty_id, price)
		PriceSet(AccountId, KittyIndex, Option<Balance>),
		/// A kitty is sold. (from, to, kitty_id, price)
		Sold(AccountId, AccountId, KittyIndex, Balance),
	}
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as Kitties {
        /// Stores all the kitties, key is the kitty id / index
        pub Kitties get(kitty): map T::KittyIndex => Option<Kitty>;
        /// Stores the total number of kitties. i.e. the next kitty index
        pub KittiesCount get(kitties_count): T::KittyIndex;

        /// Get kitty owner
        pub KittyOwners get(kitty_owner): map T::KittyIndex => Option<T::AccountId>;

        /// Get kitty price. None means not for sale.
        pub KittyPrices get(kitty_price): map T::KittyIndex => Option<BalanceOf<T>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new kitty
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let kitty_id = Self::next_kitty_id()?;

            // Generate a random 128bit value
            let dna = Self::random_value(&sender);

            // Create and store kitty
            let kitty = Kitty(dna);
            Self::insert_kitty(&sender, kitty_id, kitty);

            Self::deposit_event(RawEvent::Created(sender, kitty_id));
        }

        // /// Transfer a kitty to new owner
        // pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex) {
        //     let sender = ensure_signed(origin)?;

        //     ensure!(Self::kitty_owner(kitty_id) == Some(sender.clone()), "Only owner can transfer kitty");

        //     Self::update_owner(&to, kitty_id);

        //     Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id));
        // }

        // /// Breed kitties
        // pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) {
        //     let sender = ensure_signed(origin)?;

        //     let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;

        //     Self::deposit_event(RawEvent::Created(sender, new_kitty_id));
        // }

        // /// Set a price for a kitty for sale
        // /// None to delist the kitty
        // pub fn set_price(origin, kitty_id: T::KittyIndex, price: Option<BalanceOf<T>>) {
        //     let sender = ensure_signed(origin)?;

        //     ensure!(Self::kitty_owner(kitty_id) == Some(sender.clone()), "Only owner can set price for kitty");

        //     if let Some(ref price) = price {
        //         <KittyPrices<T>>::insert(kitty_id, price);
        //     } else {
        //         <KittyPrices<T>>::remove(kitty_id);
        //     }

        //     Self::deposit_event(RawEvent::PriceSet(sender, kitty_id, price));
        // }

        // /// Buy a kitty with max price willing to pay
        // pub fn buy(origin, kitty_id: T::KittyIndex, price: BalanceOf<T>) {
        //     let sender = ensure_signed(origin)?;

        //     let owner = Self::kitty_owner(kitty_id);
        //     ensure!(owner.is_some(), "Kitty does not exist");
        //     let owner = owner.unwrap();

        //     let kitty_price = Self::kitty_price(kitty_id);
        //     ensure!(kitty_price.is_some(), "Kitty not for sale");

        //     let kitty_price = kitty_price.unwrap();
        //     ensure!(price >= kitty_price, "Price is too low");

        //     T::Currency::transfer(&sender, &owner, kitty_price)?;

        //     <KittyPrices<T>>::remove(kitty_id);

        //     Self::update_owner(&sender, kitty_id);

        //     Self::deposit_event(RawEvent::Sold(owner, sender, kitty_id, kitty_price));
        // }
    }
}

impl<T: Trait> Module<T> {
    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = (
            T::Randomness::random(&[0]),
            sender,
            <system::Module<T>>::extrinsic_index(),
            <system::Module<T>>::block_number(),
        );
        payload.using_encoded(blake2_128)
    }

    fn next_kitty_id() -> Result<T::KittyIndex, &'static str> {
        let kitty_id = Self::kitties_count();
        if kitty_id == <T::KittyIndex as Bounded>::max_value() {
            return Err("Kitties count overflow");
        }
        Ok(kitty_id)
    }

    fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty) {
        // Create and store kitty
        <Kitties<T>>::insert(kitty_id, kitty);
        <KittiesCount<T>>::put(kitty_id + One::one());
        <KittyOwners<T>>::insert(kitty_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, kitty_id: T::KittyIndex) {
        <KittyOwners<T>>::insert(kitty_id, to);
    }

    fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
        if selector % 2 == 0 {
            return dna1;
        }

        dna2
    }

    fn do_breed(
        sender: &T::AccountId,
        kitty_id_1: T::KittyIndex,
        kitty_id_2: T::KittyIndex,
    ) -> Result<T::KittyIndex, &'static str> {
        let kitty1 = Self::kitty(kitty_id_1);
        let kitty2 = Self::kitty(kitty_id_2);

        ensure!(kitty1.is_some(), "Invalid kitty_id_1");
        ensure!(kitty2.is_some(), "Invalid kitty_id_2");
        ensure!(kitty_id_1 != kitty_id_2, "Needs different parent");
        ensure!(
            Self::kitty_owner(&kitty_id_1)
                .map(|owner| owner == *sender)
                .unwrap_or(false),
            "Not owner of kitty1"
        );
        ensure!(
            Self::kitty_owner(&kitty_id_2)
                .map(|owner| owner == *sender)
                .unwrap_or(false),
            "Not owner of kitty2"
        );

        let kitty_id = Self::next_kitty_id()?;

        let kitty1_dna = kitty1.unwrap().0;
        let kitty2_dna = kitty2.unwrap().0;

        // Generate a random 128bit value
        let selector = Self::random_value(&sender);
        let mut new_dna = [0u8; 16];

        // Combine parents and selector to create new kitty
        for i in 0..kitty1_dna.len() {
            new_dna[i] = Self::combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
        }

        Self::insert_kitty(sender, kitty_id, Kitty(new_dna));

        Ok(kitty_id)
    }
}