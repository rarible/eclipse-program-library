use anchor_lang::{
    accounts::account::Account,
    prelude::*,
};
use crate::{EditionsControls, MinterStats, Phase};

pub fn check_phase_constraints(
    phase: &Phase,
    minter_stats: &mut Account<MinterStats>,
    minter_stats_phase: &mut Account<MinterStats>,
    editions_controls: &Account<EditionsControls>,
) {
    let clock = Clock::get().unwrap();
    let current_time = clock.unix_timestamp;

    if !phase.active {
        panic!("Phase not active")
    }

    if phase.start_time > current_time {
        panic!("Phase not yet started")
    }

    if phase.end_time <= current_time {
        panic!("Phase already finished")
    }

    /// Checks if the total mints for the phase has been exceeded (phase sold out)
    /// @dev dev: notice that if max_mints_total is 0, this constraint is disabled
    if phase.max_mints_total > 0 && phase.current_mints >= phase.max_mints_total {
        panic!("Exceeded max mints for this phase")
    }

    /// Checks if the user has exceeded the max mints for the deployment (across all phases!)
    /// dev: notice that if max_mints_per_wallet is 0, this constraint is disabled
    if editions_controls.max_mints_per_wallet > 0 && minter_stats.mint_count >= editions_controls.max_mints_per_wallet {
        panic!("Exceeded wallet max mints for the collection")
    }

    /// Checks if the user has exceeded the max mints for the current phase
    /// dev: notice that if max_mints_per_wallet is 0, this constraint is disabled
    if phase.max_mints_per_wallet > 0 && minter_stats_phase.mint_count >= phase.max_mints_per_wallet {
        panic!("Exceeded wallet max mints in the current phase")
    }
}
