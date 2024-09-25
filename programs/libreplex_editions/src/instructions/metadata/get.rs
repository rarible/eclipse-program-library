use anchor_lang::prelude::*;
use borsh::BorshSerialize;
use spl_token_metadata_interface::borsh::BorshDeserialize;
use spl_token_metadata_interface::state::TokenMetadata;

// Define the accounts required for the instruction
#[derive(Accounts)]
pub struct GetMetadata<'info> {
    /// CHECK: We're only reading data from this account
    pub metadata_account: AccountInfo<'info>,
}

pub fn handler(ctx: GetMetadata) -> Result<TokenMetadata> {
    let metadata_account_info = &ctx.metadata_account;

    // Load the metadata account data
    let metadata_data = &metadata_account_info.try_borrow_data()?;

    // Parse the metadata
    let token_metadata = TokenMetadata::try_from_slice(metadata_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(token_metadata)
}
