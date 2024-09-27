use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PlatformFeeRecipient {
    pub address: Pubkey,
    pub share: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdatePlatformFeeArgs {
    pub platform_fee_value: u64, // Always required
    pub recipients: Vec<PlatformFeeRecipient>,
    pub is_fee_flat: bool, // Flag to indicate if the fee is flat
}

pub mod add;
pub mod modify;
mod get;

pub use add::*;
pub use modify::*;
