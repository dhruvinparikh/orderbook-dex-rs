/// Deprecated storages used for migration from v1.0.0 to v2.0.0 only.
use crate::{BalanceOf, Exposure, MomentOf, SessionIndex, Trait, UnlockChunk};
use codec::{Decode, Encode, HasCompact};
use frame_support::{decl_module, decl_storage};
use sp_std::prelude::*;

/// Reward points of an era. Used to split era total payout between validators.
#[derive(Encode, Decode, Default)]
pub struct EraPoints {
    /// Total number of points. Equals the sum of reward points for each validator.
    pub total: u32,
    /// The reward points earned by a given validator. The index of this vec corresponds to the
    /// index into the current validator set.
    pub individual: Vec<u32>,
}

#[derive(Encode, Decode)]
pub struct OldStakingLedger<AccountId, Balance: HasCompact> {
    pub stash: AccountId,
    #[codec(compact)]
    pub total: Balance,
    #[codec(compact)]
    pub active: Balance,
    pub unlocking: Vec<UnlockChunk<Balance>>,
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin { }
}

decl_storage! {
    pub trait Store for Module<T: Trait> as Staking {
        pub SlotStake: BalanceOf<T>;

        /// The currently elected validator set keyed by stash account ID.
        pub CurrentElected: Vec<T::AccountId>;

        /// The start of the current era.
        pub CurrentEraStart: MomentOf<T>;

        /// The session index at which the current era started.
        pub CurrentEraStartSessionIndex: SessionIndex;

        /// Rewards for the current era. Using indices of current elected set.
        pub CurrentEraPointsEarned: EraPoints;

        /// Nominators for a particular account that is in action right now. You can't iterate
        /// through validators here, but you can find them in the Session module.
        ///
        /// This is keyed by the stash account.
        pub Stakers: map hasher(blake2_256) T::AccountId => Exposure<T::AccountId, BalanceOf<T>>;

        /// Old upgrade flag.
        pub IsUpgraded: bool;
    }
}
