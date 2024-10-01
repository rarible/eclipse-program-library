use anchor_lang::{
    prelude::*,
    solana_program::{entrypoint::ProgramResult, program::invoke_signed},
};

use anchor_spl::token_interface::{
    spl_token_2022::{
        extension::{BaseStateWithExtensions, StateWithExtensions},
        state::Mint as BaseStateMint,
    },
    spl_token_metadata_interface::instruction::remove_key,
    spl_token_metadata_interface::state::Field,
    spl_token_metadata_interface::state::TokenMetadata,
    token_metadata_update_field, Mint, Token2022, TokenMetadataUpdateField,
};

use crate::errors::MetadataErrors;
use crate::utils::update_account_lamports_to_minimum_balance;
use crate::{
    EditionsDeployment, UpdatePlatformFeeArgs,
    PLATFORM_FEE_PREFIX_KEY, PLATFORM_FEE_VALUE_KEY,
};

#[derive(Accounts)]
#[instruction(args: UpdatePlatformFeeArgs)]
pub struct ModifyPlatformFee<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        seeds = ["editions_deployment".as_ref(), editions_deployment.symbol.as_ref()],
        bump
    )]
    pub editions_deployment: Account<'info, EditionsDeployment>,
    #[account(
        mut,
        constraint = signer.key() == editions_deployment.creator
    )]
    pub signer: Signer<'info>,

    #[account(
        mut,
        mint::token_program = token_program,
        constraint = editions_deployment.group_mint == group_mint.key()
    )]
    pub group_mint: Box<InterfaceAccount<'info, Mint>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
}

impl<'info> ModifyPlatformFee<'info> {
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
            metadata: self.group_mint.to_account_info().clone(),
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

    fn remove_token_metadata_field(&self, field: &str, bump_edition: u8) -> Result<()> {
        let deployment_seeds: &[&[u8]] = &[
            b"editions_deployment",
            self.editions_deployment.symbol.as_ref(),
            &[bump_edition],
        ];
        let signer_seeds: &[&[&[u8]]] = &[deployment_seeds];
        invoke_signed(
            &remove_key(
                &self.token_program.key(),
                &self.group_mint.key(),
                &self.editions_deployment.key(),
                field.to_string(),
                false,
            ),
            &[
                self.group_mint.to_account_info(),
                self.editions_deployment.to_account_info(),
            ],
            signer_seeds,
        )?;

        Ok(())
    }
}

pub fn handler(ctx: Context<ModifyPlatformFee>, args: UpdatePlatformFeeArgs) -> Result<()> {
    // Log start of handler execution
    msg!("PlatformFee::handler::start");

    // Log fetching metadata from mint account
    msg!("PlatformFee::handler::fetch_metadata");
    let metadata = {
        let mint_account = ctx.accounts.group_mint.to_account_info().clone();
        let mint_account_data = mint_account.try_borrow_data()?;
        let mint_data = StateWithExtensions::<BaseStateMint>::unpack(&mint_account_data)?;
        mint_data.get_variable_len_extension::<TokenMetadata>()?
    };

    // Validate that if fee is not flat, platform_fee_value <= 10000 (100%)
    if !args.is_fee_flat {
        require!(
            args.platform_fee_value <= 10000,
            MetadataErrors::RoyaltyBasisPointsInvalid
        );
    }

    // Update the is_fee_flat flag in metadata
    ctx.accounts.update_token_metadata_field(
        Field::Key(format!("{}is_fee_flat", PLATFORM_FEE_PREFIX_KEY)),
        args.is_fee_flat.to_string(),
        ctx.bumps.editions_deployment,
    )?;

    // Update the platform_fee_value in metadata
    ctx.accounts.update_token_metadata_field(
        Field::Key(format!("{}{}", PLATFORM_FEE_PREFIX_KEY, PLATFORM_FEE_VALUE_KEY)),
        args.platform_fee_value.to_string(),
        ctx.bumps.editions_deployment,
    )?;

    let mut total_share: u8 = 0;

    // Update recipients' metadata
    for (index, recipient) in args.recipients.iter().enumerate() {
        total_share = total_share
            .checked_add(recipient.share)
            .ok_or(MetadataErrors::RecipientShareInvalid)?;

        // Define keys with prefix and hierarchical structure
        let address_key = format!("{}recipients__{}__address", PLATFORM_FEE_PREFIX_KEY, index);
        let share_key = format!("{}recipients__{}__share", PLATFORM_FEE_PREFIX_KEY, index);

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

    // Remove old metadata fields not present in the new recipients
    msg!("PlatformFee::handler::remove_unused_metadata");
    let recipients = &args.recipients;
    let platform_fee_prefix = PLATFORM_FEE_PREFIX_KEY.to_string();
    let keys_to_remove: Vec<String> = metadata
        .additional_metadata
        .iter()
        .filter_map(|(key, _)| {
            if key.starts_with(&platform_fee_prefix) {
                let key_suffix = &key[platform_fee_prefix.len()..];
                let parts: Vec<&str> = key_suffix.split("__").collect();
                if parts.len() == 3 && parts[0] == "recipients" {
                    match parts[1].parse::<usize>() {
                        Ok(index) => {
                            if index >= recipients.len() {
                                Some(key.clone())
                            } else {
                                None
                            }
                        }
                        Err(_) => {
                            // Can't parse index, consider removing key
                            Some(key.clone())
                        }
                    }
                } else if parts.len() == 1 && (parts[0] == "is_fee_flat" || parts[0] == PLATFORM_FEE_VALUE_KEY) {
                    // Do not remove is_fee_flat or platform_fee_value
                    None
                } else {
                    // Key not recognized, consider removing
                    Some(key.clone())
                }
            } else {
                None
            }
        })
        .collect();

    for key in keys_to_remove {
        // Log removal of old metadata field
        msg!("PlatformFee::handler::remove_field: {}", key);
        ctx.accounts
            .remove_token_metadata_field(&key, ctx.bumps.editions_deployment)?;
    }

    // Update account lamports to minimum balance
    msg!("PlatformFee::handler::update_account_lamports");
    update_account_lamports_to_minimum_balance(
        ctx.accounts.group_mint.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
    )?;

    // Log successful completion of the handler
    msg!("PlatformFee::handler::success");

    Ok(())
}
