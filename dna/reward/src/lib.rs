#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod test;

use frame_support::traits::Currency;
use sp_runtime::traits::Zero;
use sp_std::prelude::*;

use frame_support::{decl_event, decl_module, decl_storage};
use frame_system::{self as system};
pub type BalanceOf<T> = <<T as pallet_staking::Trait>::Currency as Currency<
    <T as frame_system::Trait>::AccountId,
>>::Balance;

pub trait Trait: pallet_staking::Trait + pallet_treasury::Trait + pallet_balances::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    /// The account balance
    type Currency: Currency<Self::AccountId>;
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;
        /// Mint money for the treasury!
        fn on_finalize(_n: T::BlockNumber) {
            if <frame_system::Module<T>>::block_number() % Self::minting_interval() == Zero::zero() {
                let reward = Self::current_payout();
                <T as pallet_staking::Trait>::Currency::deposit_creating(&<pallet_treasury::Module<T>>::account_id(), reward);
                Self::deposit_event(RawEvent::TreasuryMinting(
                    <pallet_balances::Module<T>>::free_balance(<pallet_treasury::Module<T>>::account_id()),
                    <frame_system::Module<T>>::block_number(),
                    <pallet_treasury::Module<T>>::account_id())
                );
            }
        }
    }
}

decl_event!(
	pub enum Event<T> where <T as frame_system::Trait>::BlockNumber,
							<T as frame_system::Trait>::AccountId,
							Balance = <T as pallet_balances::Trait>::Balance {
		TreasuryMinting(Balance, BlockNumber, AccountId),
	}
);

decl_storage! {
    trait Store for Module<T: Trait> as Reward {
        /// Interval in number of blocks to reward treasury
        pub MintingInterval get(fn minting_interval) config(): T::BlockNumber;
        /// Current payout of module
        pub CurrentPayout get(fn current_payout) config(): BalanceOf<T>;
    }

    // add_extra_genesis {
    // 	config(authorities): Vec<(AuthorityId, BabeAuthorityWeight)>;
    // 	// build(|config| Module::<T>::initialize_authorities(&config.authorities))
    // }
}
