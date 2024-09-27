use anchor_lang::prelude::*;

pub mod logic;
pub use logic::*;

pub mod instructions;
pub use instructions::*;
declare_id!("J87ZtbZvSYaBhwwDb4LqCHAh7rAUjkVWiab5AZ7R3TY5");

pub mod errors;
pub mod state;

pub use state::*;

#[program]
pub mod libreplex_editions_controls {
    use libreplex_editions::{ModifyPlatformFee, UpdatePlatformFeeArgs};
    use super::*;

    // v2 endpoints. Prefer these over the original ones.
    // they allow setting of optional creator co-signer
    // and toggling inscriptions on and off.
    // for now, creator co-sign is disabled but will be enabled
    // soon to allow for wrapper contracts
    pub fn initialise_editions_controls(
        ctx: Context<InitialiseEditionControlsCtx>,
        input: InitialiseControlInput
    ) -> Result<()> {
        instructions::initialise_editions_controls(ctx, input)
    }

    pub fn add_phase(
        ctx: Context<AddPhaseCtx>,
        input: InitialisePhaseInput) -> Result<()> {
        instructions::add_phase(ctx, input)
    }

    pub fn mint_with_controls<'info>(ctx: Context<'_, '_, '_, 'info, MintWithControlsCtx<'info>>, mint_input: MintInput) -> Result<()> {
        instructions::mint_with_controls(ctx, mint_input)
    }

    pub fn modify_royalties(
        ctx: Context<UpdateRoyaltiesCtx>,
        input: libreplex_editions::UpdateRoyaltiesArgs) -> Result<()> {
        instructions::update_royalties(ctx, input)
    }

    pub fn modify_platform_fee(
        ctx: Context<UpdatePlatformFeeCtx>,
        input: libreplex_editions::UpdatePlatformFeeArgs) -> Result<()> {
        instructions::update_platform_fee(ctx, input)
    }
}
