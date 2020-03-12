//! # reward-debug-log module
//!

#![cfg_attr(not(feature = "std"), no_std)]
// The above line is needed to compile the Wasm binaries.

// Importing crates declared in the cargo.toml file.
use core::convert::{TryFrom, TryInto};
use primitives::U256;
use rstd::if_std;
use rstd::{ops::Not, prelude::*, result};
use sp_application_crypto::RuntimeAppPublic;
use sp_runtime::traits::{Bounded, CheckedSub, Hash, Member, SimpleArithmetic, Zero};
use sp_runtime::RuntimeDebug;
use support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{Get, Randomness},
    Parameter, StorageMap, StorageValue,
};

use byteorder::{ByteOrder, LittleEndian};
use codec::{Decode, Encode, EncodeLike};
use log::{debug, error, info, trace};
use system::ensure_signed;

pub trait Trait: system::Trait + session::historical::Trait {
    // type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// The identifier type for an authority.
    type AuthorityId: Member + Parameter + RuntimeAppPublic + Default + Ord;

    /// A dispatchable call type.
    type Call: From<Call<Self>>;
}

// pub trait Trait: system::Trait + im_online::Trait {
//     /// A dispatchable call type.
// 	type Call: From<Call<Self>>;
// }

decl_error! {
/// Error for the reward-debug-log module.
pub enum Error for Module<T: Trait> {
    //
}
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as RewardDebugLogStorage {
        //
    }
}

// This module's dispatchable functions.
decl_module! {
    // The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin
    {

        // pub fn deposit_event() = default;
        // type Error = Error<T>;

       /// Report some misbehavior.
        fn report_misbehavior(origin, _report: Vec<u8>) {
            ensure_signed(origin)?;
        }

        fn on_finalize(block_number: T::BlockNumber) {
            // ignore
        }
    }
}

pub mod sr25519 {
	mod app_sr25519 {
		use sp_application_crypto::{app_crypto, key_types::BABE,sr25519};
		app_crypto!(sr25519,REWARD_DEBUG_LOG);
    }

	/// An i'm online identifier using sr25519 as its crypto.
	pub type AuthorityId = app_sr25519::Public;
}
// Keep track of number of authored blocks per authority, uncles are counted as
/// well since they're a valid proof of being online.
impl<T: Trait + authorship::Trait> authorship::EventHandler<T::ValidatorId, T::BlockNumber> for Module<T> {
	fn note_author(author: T::ValidatorId) {
		// Self::note_authorship(author);
	}

	fn note_uncle(author: T::ValidatorId, _age: T::BlockNumber) {
		// Self::note_authorship(author);
	}
}

impl<T: Trait> sp_runtime::BoundToRuntimeAppPublic for Module<T> {
    type Public = T::AuthorityId;
}

impl<T: Trait> session::OneSessionHandler<T::AccountId> for Module<T> {
    type Key = T::AuthorityId;

    fn on_genesis_session<'a, I: 'a>(validators: I)
    where
        I: Iterator<Item = (&'a T::AccountId, T::AuthorityId)>,
    {
        // ignore
    }

    fn on_new_session<'a, I: 'a>(_changed: bool, validators: I, _queued_validators: I)
    where
        I: Iterator<Item = (&'a T::AccountId, T::AuthorityId)>,
    {
        // ignore
    }

    fn on_before_session_ending() {
        let current_validators = <session::Module<T>>::validators();
        info!("---Current validator reward debug log : ");
        for validator in &current_validators {
            info!("{:?}", validator);
        }
    }

    fn on_disabled(_i: usize) {
        // ignore
    }
}
