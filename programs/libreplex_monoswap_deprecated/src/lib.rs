use anchor_lang::prelude::*;

declare_id!("HPhPBUi8eSYkQb5CUUxXdeaVVwbC7xQz3a6jmkLhy9a3");

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;
pub use errors::*;

pub use constants::*;
pub use state::*;
pub use instructions::*;

#[program]
pub mod libreplex_monoswap {
    use super::*;

    pub fn create_monoswap(
        ctx: Context<CreateMonoSwapCtx>,
        input: CreateMonoSwapInput,
    ) -> Result<()> {
        instructions::create_monoswap::create_swap(ctx, input)
    }


    pub fn swap(
        ctx: Context<SwapCtx>,
    ) -> Result<()> {
        instructions::swap::swap(ctx)
    }

}
