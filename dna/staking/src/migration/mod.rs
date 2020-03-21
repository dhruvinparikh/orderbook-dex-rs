//! Update storage from v1.0.0 to v2.0.0
//!
//! In old version the staking module has several issue about handling session delay, the
//! current era was always considered the active one.
//!
//! After the migration the current era will still be considered the active one for the era of
//! the upgrade. And the delay issue will be fixed when planning the next era.
// * create:
//   * ActiveEraStart
//   * ErasRewardPoints
//   * ActiveEra
//   * ErasStakers
//   * ErasStakersClipped
//   * ErasValidatorPrefs
//   * ErasTotalStake
//   * ErasStartSessionIndex
// * translate StakingLedger
// * removal of:
//   * Stakers
//   * SlotStake
//   * CurrentElected
//   * CurrentEraStart
//   * CurrentEraStartSessionIndex
//   * CurrentEraPointsEarned

use super::*;
mod deprecated;
#[cfg(test)]
mod test_upgrade_from_master_dataset;
#[cfg(test)]
mod tests;

pub fn on_runtime_upgrade<T: Trait>() {
    match StorageVersion::get() {
        Releases::V2_0_0 => return,
        Releases::V1_0_0 => upgrade_v1_to_v2::<T>(),
    }
}

fn upgrade_v1_to_v2<T: Trait>() {
    deprecated::IsUpgraded::kill();

    let current_era_start_index = deprecated::CurrentEraStartSessionIndex::get();
    let current_era = <Module<T> as Store>::CurrentEra::get().unwrap_or(0);
    let current_era_start = deprecated::CurrentEraStart::<T>::get();
    <Module<T> as Store>::ErasStartSessionIndex::insert(current_era, current_era_start_index);
    <Module<T> as Store>::ActiveEra::put(ActiveEraInfo {
        index: current_era,
        start: Some(current_era_start),
    });

    let current_elected = deprecated::CurrentElected::<T>::get();
    let mut current_total_stake = <BalanceOf<T>>::zero();
    for validator in &current_elected {
        let exposure = deprecated::Stakers::<T>::get(validator);
        current_total_stake += exposure.total;
        <Module<T> as Store>::ErasStakers::insert(current_era, validator, &exposure);

        let mut exposure_clipped = exposure;
        let clipped_max_len = T::MaxNominatorRewardedPerValidator::get() as usize;
        if exposure_clipped.others.len() > clipped_max_len {
            exposure_clipped
                .others
                .sort_unstable_by(|a, b| a.value.cmp(&b.value).reverse());
            exposure_clipped.others.truncate(clipped_max_len);
        }
        <Module<T> as Store>::ErasStakersClipped::insert(current_era, validator, exposure_clipped);

        let pref = <Module<T> as Store>::Validators::get(validator);
        <Module<T> as Store>::ErasValidatorPrefs::insert(current_era, validator, pref);
    }
    <Module<T> as Store>::ErasTotalStake::insert(current_era, current_total_stake);

    let points = deprecated::CurrentEraPointsEarned::get();
    <Module<T> as Store>::ErasRewardPoints::insert(
        current_era,
        EraRewardPoints {
            total: points.total,
            individual: current_elected
                .iter()
                .cloned()
                .zip(points.individual.iter().cloned())
                .collect(),
        },
    );

    let res = <Module<T> as Store>::Ledger::translate_values(
        |old: deprecated::OldStakingLedger<T::AccountId, BalanceOf<T>>| StakingLedger {
            stash: old.stash,
            total: old.total,
            active: old.active,
            unlocking: old.unlocking,
            last_reward: None,
        },
    );
    if let Err(e) = res {
        frame_support::print("Encountered error in migration of Staking::Ledger map.");
        frame_support::print("The number of removed key/value is:");
        frame_support::print(e);
    }

    // Kill old storages
    deprecated::Stakers::<T>::remove_all();
    deprecated::SlotStake::<T>::kill();
    deprecated::CurrentElected::<T>::kill();
    deprecated::CurrentEraStart::<T>::kill();
    deprecated::CurrentEraStartSessionIndex::kill();
    deprecated::CurrentEraPointsEarned::kill();

    StorageVersion::put(Releases::V2_0_0);
}
