use anchor_lang::{prelude::*};
use libreplex_editions::{EditionsDeployment, UpdatePlatformFeeArgs};
use libreplex_editions::program::LibreplexEditions;
use anchor_spl::token_interface::{Mint};
use libreplex_editions::cpi::accounts::ModifyPlatformFee;
use crate::EditionsControls;

#[derive(Accounts)]
#[instruction(input: UpdatePlatformFeeArgs)]
pub struct UpdatePlatformFeeCtx<'info> {
    #[account(mut)]
    pub editions_deployment: Box<Account<'info, EditionsDeployment>>,

    #[account(mut,
        seeds = [b"editions_controls", editions_deployment.key().as_ref()],
        bump
    )]
    pub editions_controls: Box<Account<'info, EditionsControls>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    // can be different from payer for PDA integration
    #[account(mut,
        constraint = editions_controls.platform_fee_primary_admin == creator.key() ||
                     editions_controls.platform_fee_secondary_admin == creator.key())]
    pub creator: Signer<'info>,

    #[account(
        mut,
        mint::token_program = token_program,
        constraint = editions_deployment.group_mint == group_mint.key(),
    )]
    pub group_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account()]
    pub system_program: Program<'info, System>,

    /// CHECK: address checked
    #[account(address = spl_token_2022::ID)]
    pub token_program: AccountInfo<'info>,

    pub libreplex_editions_program: Program<'info, LibreplexEditions>

}

pub fn update_platform_fee(ctx: Context<UpdatePlatformFeeCtx>, platform_fee_input: UpdatePlatformFeeArgs) -> Result<()> {

    let editions_controls = &mut ctx.accounts.editions_controls;
    let libreplex_editions_program = &ctx.accounts.libreplex_editions_program;
    let editions_deployment = &ctx.accounts.editions_deployment;
    let payer = &ctx.accounts.payer;
    let mint = &ctx.accounts.group_mint;
    let system_program = &ctx.accounts.system_program;
    let token_program = &ctx.accounts.token_program;
    let platform_fee_value = platform_fee_input.platform_fee_value;
    let is_fee_flat = platform_fee_input.is_fee_flat;

    let editions_deployment_key = editions_deployment.key();
    let seeds = &[
        b"editions_controls",
        editions_deployment_key.as_ref(),
        &[ctx.bumps.editions_controls],
    ];
    msg!("libreplex_editions::cpi::modify_platform_ffe start");
    libreplex_editions::cpi::modify_platform_fee(
        CpiContext::new_with_signer(
            libreplex_editions_program.to_account_info(),
            ModifyPlatformFee {
                editions_deployment: editions_deployment.to_account_info(),
                payer: payer.to_account_info(),
                signer: editions_controls.to_account_info(),
                group_mint: mint.to_account_info(),
                token_program: token_program.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[seeds]
        ), platform_fee_input.clone())?;
    let editions_controls = &mut ctx.accounts.editions_controls;

    // Initialize an array of 5 PlatformFeeRecipient with default values
    let mut recipients_array: [libreplex_editions::PlatformFeeRecipient; 5] = [
        libreplex_editions::PlatformFeeRecipient {
            address: Pubkey::default(),
            share: 0,
        },
        libreplex_editions::PlatformFeeRecipient {
            address: Pubkey::default(),
            share: 0,
        },
        libreplex_editions::PlatformFeeRecipient {
            address: Pubkey::default(),
            share: 0,
        },
        libreplex_editions::PlatformFeeRecipient {
            address: Pubkey::default(),
            share: 0,
        },
        libreplex_editions::PlatformFeeRecipient {
            address: Pubkey::default(),
            share: 0,
        },
    ];

    msg!("libreplex_editions_controls:: update editions_controls");
    // Populate the array with provided recipients
    for (i, recipient) in platform_fee_input.recipients.iter().enumerate() {
        recipients_array[i] = recipient.clone();
    }
    editions_controls.platform_fee_value = platform_fee_value;
    editions_controls.is_fee_flat = is_fee_flat;
    editions_controls.platform_fee_recipients = recipients_array;

    msg!("libreplex_editions::cpi::modify_platform_fee done");


    Ok(())
}
