//! This module expose one function `P_NPoS` (Payout NPoS) or `compute_total_payout` which returns
//! the total payout for the era given the era duration and the staking rate in NPoS.
//! The staking rate in NPoS is the total amount of tokens staked by nominators and validators,
//! divided by the total token supply.

use sp_runtime::{traits::SimpleArithmetic, Perbill};

/// The total payout to all validators (and their nominators) per era.
///
/// Defined as such:
/// `payout = yearly_inflation(npos_token_staked / total_tokens) * total_tokans / era_per_year`
///
/// `era_duration` is expressed in millisecond.
pub fn compute_total_payout<N>(
    total_tokens: N,
    era_duration: u64,
) -> (N, N)
where
    N: SimpleArithmetic + Clone + From<u32>,
{
    // Milliseconds per year for the Julian year (365.25 days).
    const MILLISECONDS_PER_YEAR: u64 = 1000 * 3600 * 24 * 36525 / 100;

    let portion = Perbill::from_rational_approximation(era_duration as u64, MILLISECONDS_PER_YEAR);
    let payout = portion * total_tokens;

    (payout.clone(), payout)
}

