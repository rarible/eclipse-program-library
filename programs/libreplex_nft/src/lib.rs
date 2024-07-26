
use anchor_lang::prelude::*;

use instructions::*;

pub mod instructions;
pub mod errors;
pub mod state;

declare_id!("5xF73z82K2Hwn3Dk66hpGu7vVEawdc8RtNNTGVrhmeJY");

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

