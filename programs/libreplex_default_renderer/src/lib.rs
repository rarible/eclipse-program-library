use anchor_lang::prelude::*;
use instructions::*;


use anchor_lang::{AnchorDeserialize, AnchorSerialize};



pub mod instructions;



declare_id!("367qzSpbf5F8ouEJyNVJ1j7ikDxek4C25w5dqfL4eoCP");


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
