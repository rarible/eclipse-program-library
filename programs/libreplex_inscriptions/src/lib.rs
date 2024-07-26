use anchor_lang::prelude::*;
use instructions::*;


use anchor_lang::{AnchorDeserialize, AnchorSerialize};

declare_id!("5z4SW24pPdEG33W4Ud7Dn7qQhs6YzJifbS9icBhGnn3E");

pub mod instructions;
pub mod state;
pub mod constants;

pub mod errors;
pub use constants::*;

pub use state::*;

#[program]
pub mod libreplex_inscriptions {

    
    use super::*;

    pub fn create_inscription_rank_page(
        ctx: Context<CreateInscriptionRank>,
        input: CreateInscriptionRankInput,
    ) -> Result<()> {
        instructions::create_inscription_rank_page::handler(
            ctx,
            input
        )
    }

    pub fn make_inscription_immutable(
        ctx: Context<MakeInscriptionImmutable>
    ) -> Result<()> {
        instructions::make_inscription_immutable::handler(
            ctx
        )
    }

   
    pub fn claim_excess_rent(
        ctx: Context<ClaimExcessRent>,
    ) -> Result<()> {
        instructions::claim_excess_rent::handler(
            ctx
        )
    }


    pub fn migrate_to_v3(
        ctx: Context<MigrateToV3>,
    ) -> Result<()> {
        instructions::migrate_to_v3::handler(
            ctx
        )
    }

    pub fn set_validation_hash(
        ctx: Context<SetValidationHash>,
        validation_hash: Option<String>
    ) -> Result<()> {
        instructions::set_validation_hash::handler(
            ctx,
            validation_hash
        )
    }


    /* V3 methods  */
    pub fn create_inscription_v3(
        ctx: Context<CreateInscriptionV3>,
        inscription_input: CreateInscriptionInputV3,
    ) -> Result<()> {
        instructions::create_inscription_v3::handler(
            ctx,
            inscription_input
        )
    }

    pub fn create_ghost_root_inscription(
        ctx: Context<CreateGhostRootInscription>,
        inscription_input: CreateGhostRootInscriptionInput,
    ) -> Result<()> {
        instructions::ghost::handler(
            ctx,
            inscription_input
        )
    }

    pub fn make_inscription_immutable_v3(
        ctx: Context<MakeInscriptionImmutableV3>
    ) -> Result<()> {
        instructions::make_inscription_immutable_v3::handler(
            ctx
        )
    }


    pub fn resize_inscription_v3(
        ctx: Context<ResizeInscriptionV3>,
        input: ResizeInscriptionInput,
    ) -> Result<()> {
        instructions::resize_inscription_v3::handler(
            ctx,
            input
        )
    }

    pub fn write_to_inscription_v3(
        ctx: Context<WriteToInscriptionV3>,
        input: WriteToInscriptionInput,
    ) -> Result<()> {
        instructions::write_to_inscription_v3::handler(
            ctx,
            input
        )
    }





}