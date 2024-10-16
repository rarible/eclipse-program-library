use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{self, ID as TOKEN_2022_ID},
};
use libreplex_editions::{cpi::accounts::MintCtx, program::LibreplexEditions, EditionsDeployment};
use crate::{check_phase_constraints, errors::EditionsError, EditionsControls, MinterStats};
use libreplex_editions::group_extension_program;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MintInput {
    pub phase_index: u32,
}

#[derive(Accounts)]
#[instruction(mint_input: MintInput)]
pub struct MintWithControlsCtx<'info> {
    #[account(mut)]
    pub editions_deployment: Box<Account<'info, EditionsDeployment>>,

    #[account(
        mut,
        seeds = [b"editions_controls", editions_deployment.key().as_ref()],
        bump
    )]
    pub editions_controls: Box<Account<'info, EditionsControls>>,

    /// CHECK: Checked via CPI
    #[account(mut)]
    pub hashlist: UncheckedAccount<'info>,

    /// CHECK: Checked via CPI
    #[account(mut)]
    pub hashlist_marker: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// When deployment.require_creator_cosign is true, this must be equal to the creator
    /// of the deployment; otherwise, can be any signer account
    #[account(
        constraint = editions_controls.cosigner_program_id == system_program::ID
            || signer.key() == editions_deployment.creator
    )]
    pub signer: Signer<'info>,

    /// CHECK: Anybody can sign, anybody can receive the inscription
    #[account(mut)]
    pub minter: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        seeds = [b"minter_stats", editions_deployment.key().as_ref(), minter.key().as_ref()],
        bump,
        space = MinterStats::SIZE
    )]
    pub minter_stats: Box<Account<'info, MinterStats>>,

    #[account(
        init_if_needed,
        payer = payer,
        seeds = [
            b"minter_stats_phase",
            editions_deployment.key().as_ref(),
            minter.key().as_ref(),
            &mint_input.phase_index.to_le_bytes()
        ],
        bump,
        space = MinterStats::SIZE
    )]
    pub minter_stats_phase: Box<Account<'info, MinterStats>>,

    #[account(mut)]
    pub mint: Signer<'info>,

    #[account(mut)]
    pub member: Signer<'info>,

    /// CHECK: checked in constraint
    #[account(
        mut,
        constraint = editions_deployment.group == group.key()
    )]
    pub group: UncheckedAccount<'info>,

    /// CHECK: Checked in constraint
    #[account(
        mut,
        constraint = editions_deployment.group_mint == group_mint.key()
    )]
    pub group_mint: UncheckedAccount<'info>,

    /// CHECK: Platform fee recipient
    #[account(mut)]
    pub platform_fee_recipient_1: UncheckedAccount<'info>,

    // TODO add optimization for stack
    // #[account(mut)]
    // pub platform_fee_recipient_2: UncheckedAccount<'info>,

    /// CHECK: Passed in via CPI to mpl_token_metadata program
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,

    /// CHECK: Checked in constraint
    #[account(
        mut,
        constraint = editions_controls.treasury == treasury.key()
    )]
    pub treasury: UncheckedAccount<'info>,

    /// CHECK: Checked in constraint
    #[account(
        constraint = token_program.key() == TOKEN_2022_ID
    )]
    pub token_program: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    /// CHECK: Address checked
    #[account(address = group_extension_program::ID)]
    pub group_extension_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,

    pub libreplex_editions_program: Program<'info, LibreplexEditions>,
}

pub fn mint_with_controls(
    ctx: Context<MintWithControlsCtx>,
    mint_input: MintInput,
) -> Result<()> {
    let editions_controls = &mut ctx.accounts.editions_controls;
    let minter_stats = &mut ctx.accounts.minter_stats;
    let minter_stats_phase = &mut ctx.accounts.minter_stats_phase;
    let minter = &ctx.accounts.minter;

    // Phase validation and price retrieval
    let price_amount = validate_phase(editions_controls, mint_input.phase_index)?;

    // Check phase constraints
    check_phase_constraints(
        &editions_controls.phases[mint_input.phase_index as usize],
        minter_stats,
        minter_stats_phase,
        editions_controls,
    );

    // Update minter statistics
    update_minter_stats(
        minter_stats,
        minter_stats_phase,
        &minter.key(),
        editions_controls,
        mint_input.phase_index as usize,
    );

    // Process platform fees and transfer remaining amount to treasury
    process_platform_fees(&ctx, price_amount)?;

    // Prepare seeds for signer
    let editions_deployment_key = ctx.accounts.editions_deployment.key();
    let seeds = &[
        b"editions_controls",
        editions_deployment_key.as_ref(),
        &[ctx.bumps.editions_controls],
    ];

    // Perform the minting process
    perform_mint(&ctx, seeds)?;

    Ok(())
}

fn validate_phase(
    editions_controls: &EditionsControls,
    phase_index: u32,
) -> Result<u64> {
    if phase_index >= editions_controls.phases.len() as u32 {
        if editions_controls.phases.is_empty() {
            return Err(EditionsError::NoPhasesAdded.into());
        } else {
            return Err(EditionsError::InvalidPhaseIndex.into());
        }
    }

    let price_amount = editions_controls.phases[phase_index as usize].price_amount;
    Ok(price_amount)
}

fn update_minter_stats(
    minter_stats: &mut MinterStats,
    minter_stats_phase: &mut MinterStats,
    minter_key: &Pubkey,
    editions_controls: &mut EditionsControls,
    phase_index: usize,
) {
    minter_stats.wallet = *minter_key;
    minter_stats.mint_count = minter_stats.mint_count.saturating_add(1);

    minter_stats_phase.wallet = *minter_key;
    minter_stats_phase.mint_count = minter_stats_phase.mint_count.saturating_add(1);

    editions_controls.phases[phase_index].current_mints =
        editions_controls.phases[phase_index].current_mints.saturating_add(1);
}

fn process_platform_fees(
    ctx: &Context<MintWithControlsCtx>,
    price_amount: u64,
) -> Result<()> {
    let editions_controls = &ctx.accounts.editions_controls;
    let payer = &ctx.accounts.payer;
    let treasury = &ctx.accounts.treasury;
    let system_program = &ctx.accounts.system_program;
    let recipients = &editions_controls.platform_fee_recipients;

    // Ensure that the sum of shares equals 100
    let total_shares: u8 = recipients.iter().map(|r| r.share).sum();
    if total_shares != 100 {
        return Err(EditionsError::InvalidFeeShares.into());
    }

    let total_fee: u64;
    let remaining_amount: u64;

    if editions_controls.is_fee_flat {
        total_fee = editions_controls.platform_fee_value;
        remaining_amount = price_amount;
    } else {
        // Calculate fee as (price_amount * platform_fee_value) / 10,000 (assuming basis points)
        total_fee = price_amount
            .checked_mul(editions_controls.platform_fee_value as u64)
            .ok_or(EditionsError::FeeCalculationError)?
            .checked_div(10_000)
            .ok_or(EditionsError::FeeCalculationError)?;

        remaining_amount = price_amount.checked_sub(total_fee)
            .ok_or(EditionsError::FeeCalculationError)?;
    }

    // Distribute fees to recipients
    for (i, recipient_struct) in recipients.iter().enumerate() {
        if recipient_struct.share == 0 {
            continue;
        }

        let recipient_account = &ctx.accounts.platform_fee_recipient_1;

        msg!("check recipients {}: {} vs {}", i, recipient_account.key(), recipient_struct.address.key());
        // Ensure that the account matches the expected recipient
        if recipient_account.key() != recipient_struct.address.key() {
            return Err(EditionsError::RecipientMismatch.into());
        }

        let recipient_fee = total_fee
            .checked_mul(recipient_struct.share as u64)
            .ok_or(EditionsError::FeeCalculationError)?
            .checked_div(100)
            .ok_or(EditionsError::FeeCalculationError)?;

        // Transfer platform fee to recipient
        system_program::transfer(
            CpiContext::new(
                system_program.to_account_info(),
                system_program::Transfer {
                    from: payer.to_account_info(),
                    to: recipient_account.to_account_info(),
                },
            ),
            recipient_fee,
        )?;

        break;
    }

    // Transfer remaining amount to treasury
    system_program::transfer(
        CpiContext::new(
            system_program.to_account_info(),
            system_program::Transfer {
                from: payer.to_account_info(),
                to: treasury.to_account_info(),
            },
        ),
        remaining_amount,
    )?;

    Ok(())
}

fn perform_mint(
    ctx: &Context<MintWithControlsCtx>,
    seeds: &[&[u8]],
) -> Result<()> {
    let libreplex_editions_program = &ctx.accounts.libreplex_editions_program;
    let editions_controls = &ctx.accounts.editions_controls;

    libreplex_editions::cpi::mint(
        CpiContext::new_with_signer(
            libreplex_editions_program.to_account_info(),
            MintCtx {
                editions_deployment: ctx.accounts.editions_deployment.to_account_info(),
                hashlist: ctx.accounts.hashlist.to_account_info(),
                hashlist_marker: ctx.accounts.hashlist_marker.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                signer: editions_controls.to_account_info(),
                minter: ctx.accounts.minter.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                group: ctx.accounts.group.to_account_info(),
                group_mint: ctx.accounts.group_mint.to_account_info(),
                token_account: ctx.accounts.token_account.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                group_extension_program: ctx.accounts.group_extension_program.to_account_info(),
                member: ctx.accounts.member.to_account_info(),
            },
            &[seeds],
        ),
    )?;

    Ok(())
}
