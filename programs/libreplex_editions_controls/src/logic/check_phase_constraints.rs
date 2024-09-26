use anchor_lang::accounts::account::Account;

use anchor_lang::prelude::*;

use crate::{EditionsControls, MinterStats, Phase};

pub fn check_phase_constraints(
    phase: &Phase,
    minter_stats: &mut Account<MinterStats>,
    minter_stats_phase: &mut Account<MinterStats>,
    editions_controls: &Account<EditionsControls>,
    merkle_proof: Option<Vec<[u8; 32]>>,
    minter: &Pubkey,
) {
    // check
    let clock = Clock::get().unwrap();
    let current_time = clock.unix_timestamp;

    if !phase.active {
        panic!("Phase not active")
    }

    msg!("{} {}", phase.start_time, current_time);
    if phase.start_time > current_time {
        panic!("Phase not yet started")
    }

    if phase.end_time <= current_time {
        panic!("Phase already finished")
    }

    if phase.max_mints_per_wallet > 0 && minter_stats_phase.mint_count >= phase.max_mints_per_wallet {
        panic!("This wallet has exceeded max mints in the current phase")
    }

    if phase.max_mints_total > 0 && phase.current_mints >= phase.max_mints_total {
        panic!("Total mints exceeded in this phase")
    }

    if editions_controls.max_mints_per_wallet > 0 && minter_stats.mint_count >= editions_controls.max_mints_per_wallet {
        panic!("This wallet has exceeded max mints for the deployment")
    }
    
    // check private phase constraints @dev on-going development
    if phase.is_private {
        if let Some(merkle_root) = phase.merkle_root {
            if let Some(proof) = merkle_proof {

                // construct leaf, check PhaseTreeNode.
                let leaf = hashv(&[
                    // minter
                    // price
                    // max_claims
                ]);

                if !verify(proof, merkle_root, leaf.to_bytes()) {
                    panic!("Invalid merkle proof");
                }
            } else {
                panic!("Merkle proof required for private phase");
            }
        } else {
            panic!("Merkle root not set for private phase");
        }
    }
}
