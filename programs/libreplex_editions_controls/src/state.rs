use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;

#[derive(Clone, AnchorDeserialize, AnchorSerialize)]
pub struct Phase {
    pub price_amount: u64,
    pub price_token: Pubkey, // SO111111 - native ß
    pub start_time: i64, // set to any date before now for instant activate
    pub active: bool,
    pub max_mints_per_wallet: u64, // set to 0 for unlimited
    pub max_mints_total: u64, // set to 0 for unlimited 
    pub end_time: i64, // set to i64::MAX for unlimited
    pub current_mints: u64,
    pub merkle_root: Option<[u8; 32]>,
    pub padding: [u8; 136]
}

impl Phase {
    pub const SIZE: usize = 
    8
    + 8
    + 32
    + 8
    + 1
    + 8
    + 8
    + 8
    + 8
    + 33 
    + 136;
}

pub const DEFAULT_PLATFORM_FEE_PRIMARY_ADMIN: &str = "674s1Sap3KVnr8WGrY5KGQ69oTYjjgr1disKJo6GpTYw";
pub const DEFAULT_PLATFORM_FEE_SECONDARY_ADMIN: &str = "QjzRL6VwKGnpco8wx3cPjtq8ZPhewy7ohq7F5mv2eeR";

#[account]
pub struct MinterStats {
    pub wallet: Pubkey,
    pub mint_count: u64, 
    pub padding: [u8; 50]
}

impl MinterStats {
    pub const SIZE: usize = 8 + 32 + 8 + 50;
}

#[account]
pub struct EditionsControls {
    pub editions_deployment: Pubkey,
    pub creator: Pubkey,
    pub treasury: Pubkey, // mint proceeds go here
    pub max_mints_per_wallet: u64, // set to 0 for unlimited (applied across all the phases)
    pub cosigner_program_id: Pubkey,
    pub platform_fee_primary_admin: Pubkey,
    pub platform_fee_secondary_admin: Pubkey,
    pub phases: Vec<Phase>,
    pub padding: [u8; 136], 
}

impl EditionsControls {
    pub const INITIAL_SIZE: usize = 8 + 32 + 32 + 32 + 8 + 32 + 32 + 32 + 4 + 136;
    pub fn get_size(number_of_phases: usize) -> usize {
        EditionsControls::INITIAL_SIZE + Phase::SIZE * number_of_phases
    }
}
