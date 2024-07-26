
use anchor_lang::prelude::*;

use instructions::*;

pub mod instructions;
pub mod errors;
pub mod state;

declare_id!("3oR36M1JQ9LHM1xykKTHDvhuYVM5mDAqRmq989zc1Pzi");

#[program]
pub mod libreplex_nft {
    
    use super::*;

    pub fn wrap(
        ctx: Context<WrapCtx>,
    ) -> Result<()> {
        instructions::wrap::handler(ctx)
    }

    pub fn toggle_freeze(
        ctx: Context<ToggleFreezeCtx>,
        input: ToggleFreezeInput,
    ) -> Result<()> {
        instructions::toggle_freeze::handler(ctx, input)
    }

}

