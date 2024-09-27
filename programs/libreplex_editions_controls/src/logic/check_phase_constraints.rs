use anchor_lang::{
    accounts::account::Account,
    prelude::*,
    solana_program::hash::hashv,
};

use rarible_merkle_verify::verify;

use crate::{EditionsControls, MinterStats, Phase};

// We need to discern between leaf and intermediate nodes to prevent trivial second
// pre-image attacks.
// https://flawed.net.nz/2018/02/21/attacking-merkle-trees-with-a-second-preimage-attack
const LEAF_PREFIX: &[u8] = &[0];

pub fn check_phase_constraints(
    phase: &Phase,
    minter_stats: &mut Account<MinterStats>,
    minter_stats_phase: &mut Account<MinterStats>,
    editions_controls: &Account<EditionsControls>,
    merkle_proof: Option<Vec<[u8; 32]>>,
    minter: &Pubkey,
    allow_list_price: Option<u64>,
    allow_list_max_claims: Option<u64>,
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
                if let (Some(price), Some(max_claims)) = (allow_list_price, allow_list_max_claims) {
                    // construct leaf
                    let leaf = hashv(&[
                        &minter.to_bytes(),
                        &price.to_le_bytes(),
                        &max_claims.to_le_bytes(),
                    ]);

                    let node = hashv(&[LEAF_PREFIX, &leaf.to_bytes()]);

                    if !verify(proof, merkle_root, node.to_bytes()) {
                        panic!("Invalid merkle proof");
                    }
                } else {
                    panic!("Merkle proof required for private phase");
                }
            } else {
                panic!("Merkle proof required for private phase");
            }
        } else {
            panic!("Merkle root not set for private phase");
        }
    }
}
