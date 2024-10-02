use anchor_lang::prelude::*;
use instructions::*;


use anchor_lang::{AnchorDeserialize, AnchorSerialize};



pub mod instructions;



declare_id!("8MzCUXH5Y52jjorkSu9G6Y69AHBwFoW8GC7MFFhDJRaZ");


#[program]
pub mod libreplex_default_renderer {





    use super::*;
    pub fn canonical(
        ctx: Context<RenderContext>,
        render_input: RenderInput
    ) -> Result<Vec<u8>> {
        instructions::canonical::handler(ctx, render_input)
    }


}
