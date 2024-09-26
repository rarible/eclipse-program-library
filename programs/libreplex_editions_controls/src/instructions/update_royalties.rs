use anchor_lang::{prelude::*};
use libreplex_editions::{EditionsDeployment, UpdateRoyaltiesArgs};
use libreplex_editions::program::LibreplexEditions;
use anchor_spl::token_interface::{Mint};
use libreplex_editions::cpi::accounts::ModifyRoyalties;

#[derive(Accounts)]
#[instruction(input: UpdateRoyaltiesArgs)]
pub struct UpdateRoyaltiesCtx<'info> {
    #[account(mut,
        seeds = ["editions_deployment".as_ref(), editions_deployment.symbol.as_ref()], bump)]
    pub editions_deployment: Account<'info, EditionsDeployment>,

    #[account(mut)]
    pub payer: Signer<'info>,

    // can be different from payer for PDA integration
    #[account(mut,
        constraint = editions_deployment.creator == creator.key())]
    pub creator: Signer<'info>,

    #[account(
        mut,
        mint::token_program = token_program,
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,

    #[account()]
    pub system_program: Program<'info, System>,

    /// CHECK: address checked
    #[account(address = spl_token_2022::ID)]
    pub token_program: AccountInfo<'info>,

    pub libreplex_editions_program: Program<'info, LibreplexEditions>

}

pub fn update_royalties(ctx: Context<UpdateRoyaltiesCtx>, royalties_input: UpdateRoyaltiesArgs) -> Result<()> {

    let libreplex_editions_program = &ctx.accounts.libreplex_editions_program;
    let editions_deployment = &ctx.accounts.editions_deployment;
    let payer = &ctx.accounts.payer;
    let mint = &ctx.accounts.mint;
    let system_program = &ctx.accounts.system_program;
    let token_program = &ctx.accounts.token_program;

    let editions_deployment_key = editions_deployment.key();
    let seeds = &[
        b"editions_controls",
        editions_deployment_key.as_ref(),
        &[ctx.bumps.editions_deployment],
    ];

    libreplex_editions::cpi::modify_royalties(
        CpiContext::new_with_signer(
            libreplex_editions_program.to_account_info(),
            ModifyRoyalties {
                editions_deployment: editions_deployment.to_account_info(),
                payer: payer.to_account_info(),
                signer: editions_deployment.to_account_info(),
                mint: mint.to_account_info(),
                token_program: token_program.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[seeds]
        ), royalties_input)?;

    Ok(())
}
