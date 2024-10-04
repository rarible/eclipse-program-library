use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PlatformFeeRecipient {
    pub address: Pubkey,
    pub share: u8, // Share percentage (0-100)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdatePlatformFeeArgs {
    pub platform_fee_value: u64, // Always required
    pub recipients: Vec<PlatformFeeRecipient>,
    pub is_fee_flat: bool, // Flag to indicate if the fee is flat
}

impl PlatformFeeRecipient {
    pub const SIZE: usize = 8 + 32 + 1; // Pubkey + u8
}

// max length: 8 + 8 + 8 + 1 + 8 = 33
#[derive(Clone, AnchorDeserialize, AnchorSerialize)]
pub struct Phase {
    pub price_amount: u64,
    pub price_token: Pubkey, // SO111111 - native ß
    pub start_time: i64, // set to any date before now for instant activate
    pub active: bool,
    pub max_mints_per_wallet: u64, // set to 0 for unlimited
    pub max_mints_total: u64, // set to 0 for unlimited (applied across all the phases)
    pub end_time: i64, // set to i64::MAX for unlimited
    pub current_mints: u64,
    pub padding: [u8; 200]
}

impl Phase {
    pub const SIZE: usize = 8 
    + 32 
    + 8
    + 1
    + 8
    + 8
    + 8
    + 8
    + 200;
}

pub const DEFAULT_PLATFORM_FEE_PRIMARY_ADMIN: &str = "674s1Sap3KVnr8WGrY5KGQ69oTYjjgr1disKJo6GpTYw";
pub const DEFAULT_PLATFORM_FEE_SECONDARY_ADMIN: &str = "QjzRL6VwKGnpco8wx3cPjtq8ZPhewy7ohq7F5mv2eeR";

#[account]
pub struct MinterStats {
    pub wallet: Pubkey,
    pub mint_count: u64, // set to any date before now for instant activate
    pub padding: [u8; 50]
}

impl MinterStats {
    pub const SIZE: usize = 8 + 32 + 8 + 8 + 50;
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
    pub platform_fee_value: u64, // Fee amount or basis points
    pub is_fee_flat: bool, // True for flat fee, false for percentage-based fee
    pub platform_fee_recipients: [PlatformFeeRecipient; 5], // Fixed-length array of 5 recipients and their shares
    pub padding: [u8; 200],    // in case we need some more stuff in the future
    pub phases: Vec<Phase>,
}

impl EditionsControls {
    pub const INITIAL_SIZE: usize = 8          // Discriminator
        + 32                                   // editions_deployment
        + 32                                   // creator
        + 32                                   // treasury
        + 8                                    // max_mints_per_wallet
        + 32                                   // cosigner_program_id
        + 32                                   // platform_fee_primary_admin
        + 32                                   // platform_fee_secondary_admin
        + 8                                    // platform_fee_value
        + 1                                    // is_fee_flat
        + (PlatformFeeRecipient::SIZE * 5)     // platform_fee_recipients (5 * 33 = 165)
        + 200                                  // padding
        + 4;                                   // Vec length for phases
    pub fn get_size(number_of_phases: usize) -> usize {
        EditionsControls::INITIAL_SIZE + Phase::SIZE * number_of_phases
    }
}
