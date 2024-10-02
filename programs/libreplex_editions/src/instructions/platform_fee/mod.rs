use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PlatformFeeRecipient {
    pub address: Pubkey,
    pub share: u8,
}

impl PlatformFeeRecipient {
    pub const SIZE: usize = 32 + 1; // Pubkey (32 bytes) + u8 (1 byte) = 33 bytes
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdatePlatformFeeArgs {
    pub platform_fee_value: u64, // Always required
    pub recipients: Vec<PlatformFeeRecipient>,
    pub is_fee_flat: bool, // Flag to indicate if the fee is flat
}

pub mod add;
pub mod modify;
pub mod get;

pub use add::*;
pub use get::*;
pub use modify::*;
