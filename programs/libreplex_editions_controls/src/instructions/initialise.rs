use anchor_lang::{prelude::*, system_program};
use libreplex_editions::{cpi::accounts::InitialiseCtx, group_extension_program, program::LibreplexEditions, AddMetadataArgs, CreatorWithShare, InitialiseInput};
use libreplex_editions::cpi::accounts::AddMetadata;
use crate::EditionsControls;

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct InitialiseControlInput {
    pub max_mints_per_wallet: u64,
    pub treasury: Pubkey,
    pub max_number_of_tokens: u64,
    pub symbol: String,
    pub name: String,
    pub offchain_url: String,
    pub cosigner_program_id: Option<Pubkey>,
    pub royalty_basis_points: u16,
    pub creators: Vec<CreatorWithShare>,
}

#[derive(Accounts)]
#[instruction(_initialise_controls_input: InitialiseControlInput)]
pub struct InitialiseEditionControlsCtx<'info> {
    #[account(init,
        space = EditionsControls::INITIAL_SIZE,
        payer = payer,
        seeds = [b"editions_controls", editions_deployment.key().as_ref()],
        bump
    )]
    pub editions_controls: Account<'info, EditionsControls>,

    /// CHECK: CPI: Passed into libreplex_editions program for initialisation. Checking seed here for early warning
    #[account(mut)]
    pub editions_deployment: UncheckedAccount<'info>,

    /// CHECK: Checked in via CPI
    #[account(mut)]
    pub hashlist: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: can be different from payer for PDA integration
    #[account()]
    pub creator: UncheckedAccount<'info>,

    /// CHECK: created
    #[account(mut)]
    pub group_mint: Signer<'info>,

    /// CHECK: created
    #[account(mut)]
    pub group: Signer<'info>,

    #[account()]
    pub system_program: Program<'info, System>,

    /// CHECK: address checked
    #[account(address = spl_token_2022::ID)]
    pub token_program: AccountInfo<'info>,

    /// CHECK: address checked
    #[account(address = group_extension_program::ID)]
    pub group_extension_program: AccountInfo<'info>,

    pub libreplex_editions_program: Program<'info, LibreplexEditions>,
}

pub fn initialise_editions_controls(
    ctx: Context<InitialiseEditionControlsCtx>,
    input: InitialiseControlInput,
) -> Result<()> {
    let libreplex_editions_program = &ctx.accounts.libreplex_editions_program;
    let editions_controls = &mut ctx.accounts.editions_controls;

    let editions_deployment = &ctx.accounts.editions_deployment;
    let hashlist = &ctx.accounts.hashlist;
    let payer = &ctx.accounts.payer;
    let creator = &ctx.accounts.creator;
    let group = &ctx.accounts.group;
    let group_mint = &ctx.accounts.group_mint;
    let system_program = &ctx.accounts.system_program;
    let token_program = &ctx.accounts.token_program;
    let group_extension_program = &ctx.accounts.group_extension_program;

    let core_input = InitialiseInput {
        max_number_of_tokens: input.max_number_of_tokens,
        symbol: input.symbol,
        name: input.name,
        offchain_url: input.offchain_url,
        creator_cosign_program_id: Some(crate::ID),
        royalty_basis_points: input.royalty_basis_points,
        creators: input.creators,
    };

    // Initialize the editions using CPI
    libreplex_editions::cpi::initialise(
        CpiContext::new(
            libreplex_editions_program.to_account_info(),
            InitialiseCtx {
                editions_deployment: editions_deployment.to_account_info(),
                hashlist: hashlist.to_account_info(),
                payer: payer.to_account_info(),
                creator: editions_controls.to_account_info(),
                group: group.to_account_info(),
                group_mint: group_mint.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
                group_extension_program: group_extension_program.to_account_info(),
            },
        ),
        core_input,
    )?;

    // Add metadata using signer seeds
    let mut metadata: Vec<AddMetadataArgs> = vec![];
    let metadata_input = AddMetadataArgs {
        field: "test_key".to_string(),
        value: "test_value".to_string(),
    };



    // Set the editions control state
    editions_controls.set_inner(EditionsControls {
        editions_deployment: editions_deployment.key(),
        creator: creator.key(),
        max_mints_per_wallet: input.max_mints_per_wallet,
        padding: [0; 200],
        cosigner_program_id: input.cosigner_program_id.unwrap_or(system_program::ID),
        phases: vec![],
        treasury: input.treasury,
    });

    let editions_deployment_key = editions_deployment.key();
    let seeds = &[
        b"editions_controls",
        editions_deployment_key.as_ref(),
        &[ctx.bumps.editions_controls],
    ];

    // Add metadata CPI call
    metadata.push(metadata_input);
    libreplex_editions::cpi::add_metadata(
        CpiContext::new_with_signer(
            libreplex_editions_program.to_account_info(),
            AddMetadata {
                editions_deployment: editions_deployment.to_account_info(),
                payer: payer.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
                mint: group_mint.to_account_info(),
                signer: editions_controls.to_account_info(),
            },
            &[seeds]
        ),
        metadata,
    )?;

    Ok(())
}
