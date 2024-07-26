use anchor_lang::prelude::*;
use instructions::*;


use anchor_lang::{AnchorDeserialize, AnchorSerialize};



pub mod instructions;



declare_id!("Hnw68yc4RNgu1CuB3zY9gc81QCWX94CMF115gnkHHHhX");


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
