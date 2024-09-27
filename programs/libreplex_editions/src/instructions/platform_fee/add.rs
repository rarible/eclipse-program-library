use crate::errors::MetadataErrors;
use crate::utils::update_account_lamports_to_minimum_balance;
use crate::{EditionsDeployment, UpdatePlatformFeeArgs, PLATFORM_FEE_PREFIX_KEY, PLATFORM_FEE_VALUE_KEY};
use anchor_lang::system_program;
use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};
use anchor_spl::token_interface::{
    spl_token_metadata_interface::state::Field,
    token_metadata_update_field,
    Mint,
    Token2022,
    TokenMetadataUpdateField,
};

#[derive(Accounts)]
#[instruction(args: UpdatePlatformFeeArgs)]
pub struct AddPlatformFee<'info> {
    #[account(mut,
        seeds = ["editions_deployment".as_ref(), editions_deployment.symbol.as_ref()], bump)]
    pub editions_deployment: Account<'info, EditionsDeployment>,
    #[account(mut)]
    pub payer: Signer<'info>,

    // Must be equal to the creator of the deployment
    #[account(mut,
        constraint = signer.key() == editions_deployment.creator)]
    pub signer: Signer<'info>,
    #[account(mut,
        constraint = editions_deployment.group_mint == group_mint.key())]
    pub group_mint: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
}

impl<'info> AddPlatformFee<'info> {
    fn update_token_metadata_field(
        &self,
        field: Field,
        value: String,
        bump_edition: u8,
    ) -> ProgramResult {
        let deployment_seeds: &[&[u8]] = &[
            b"editions_deployment",
            self.editions_deployment.symbol.as_ref(),
            &[bump_edition],
        ];
        let signer_seeds: &[&[&[u8]]] = &[deployment_seeds];

        let cpi_accounts = TokenMetadataUpdateField {
            token_program_id: self.token_program.to_account_info(),
            metadata: self.group_mint.to_account_info(),
            update_authority: self.editions_deployment.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        token_metadata_update_field(cpi_ctx, field, value)?;
        Ok(())
    }
}

pub fn handler(ctx: Context<AddPlatformFee>, args: UpdatePlatformFeeArgs) -> Result<()> {
    // Validate that if fee is not flat, platform_fee_value <= 10000 (100%)
    if !args.is_fee_flat {
        require!(
            args.platform_fee_value <= 10000,
            MetadataErrors::RoyaltyBasisPointsInvalid
        );
    }

    // Save the is_fee_flat flag in metadata
    ctx.accounts.update_token_metadata_field(
        Field::Key(format!(
            "{}is_fee_flat",
            PLATFORM_FEE_PREFIX_KEY
        )),
        args.is_fee_flat.to_string(),
        ctx.bumps.editions_deployment,
    )?;

    // Always save the PLATFORM_FEE_VALUE in metadata
    ctx.accounts.update_token_metadata_field(
        Field::Key(format!(
            "{}{}",
            PLATFORM_FEE_PREFIX_KEY, PLATFORM_FEE_VALUE_KEY
        )),
        args.platform_fee_value.to_string(),
        ctx.bumps.editions_deployment,
    )?;

    let mut total_share: u8 = 0;

    // Add recipients and their respective shares to metadata with prefixed keys
    for (index, recipient) in args.recipients.iter().enumerate() {
        total_share = total_share
            .checked_add(recipient.share)
            .ok_or(MetadataErrors::PlatformFeeBasisPointsInvalid)?;

        // Define keys with prefix and hierarchical structure
        let address_key = format!(
            "{}recipients__{}__address",
            PLATFORM_FEE_PREFIX_KEY, index
        );
        let share_key = format!(
            "{}recipients__{}__share",
            PLATFORM_FEE_PREFIX_KEY, index
        );

        // Update metadata with the recipient's address
        ctx.accounts.update_token_metadata_field(
            Field::Key(address_key),
            recipient.address.to_string(),
            ctx.bumps.editions_deployment,
        )?;

        // Update metadata with the recipient's share
        ctx.accounts.update_token_metadata_field(
            Field::Key(share_key),
            recipient.share.to_string(),
            ctx.bumps.editions_deployment,
        )?;
    }

    if total_share != 100 {
        return Err(MetadataErrors::RecipientShareInvalid.into());
    }

    // Transfer minimum rent to mint account
    update_account_lamports_to_minimum_balance(
        ctx.accounts.group_mint.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
    )?;

    Ok(())
}
