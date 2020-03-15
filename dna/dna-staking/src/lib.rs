#![recursion_limit="128"]
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{prelude::*, result, collections::btree_map::BTreeMap};
use codec::{HasCompact, Encode, Decode};
use frame_support::{
	decl_module, decl_event, decl_storage, ensure, decl_error,
	weights::SimpleDispatchInfo,
	dispatch::DispatchResult,
	traits::{
		Currency, LockIdentifier, LockableCurrency,
		WithdrawReasons, OnUnbalanced, Imbalance, Get, Time
	}
};
use pallet_session::SessionHandler;
use sp_runtime::{
  Perbill,
  // PerThing, KP: Failed due to  no `PerThing` in the root
  RuntimeDebug,
	curve::PiecewiseLinear,
	traits::{
		Convert, Zero, StaticLookup, CheckedSub, Saturating, SaturatedConversion,
    // AtLeast32Bit, KP: Failed due to  no `AtLeast32Bit` in `traits`
    EnsureOrigin,
	}
};
use sp_staking::{
	SessionIndex,
  offence::{OnOffenceHandler, OffenceDetails, Offence, ReportOffence,
    // OffenceError KP: Failed due to no `OffenceError` in `offence`
  },
};
#[cfg(feature = "std")]
use sp_runtime::{Serialize, Deserialize};
use frame_system::{self as system, ensure_signed, ensure_root};

use sp_phragmen::ExtendedBalance;

decl_error! {

}

decl_event! {

  pub enum Event<T> where Balance = BalanceOf<T>,
    <T as frame_system::Trait>::AccountId {
      Reward(AccountId, Balance)
    }

}

decl_storage! {

}