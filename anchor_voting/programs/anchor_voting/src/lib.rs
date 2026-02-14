use anchor_lang::prelude::*;

mod state;
mod instructions;

use instructions::*;
declare_id!("2PLpck15JpdjvbTFzSegbzAUGSowxgPXtm73fm2A2hYM");

#[program]
pub mod anchor_voting {
    use super::*;

    pub fn init_dao(ctx: Context<InitDao>, name: String) -> Result<()> {
        instructions::init_dao::init_dao(ctx, name)
    }
    pub fn init_proposal(ctx: Context<InitProposal>, metadata: String) -> Result<()> {
        instructions::init_proposal::init_proposal(ctx, metadata)
    }

    pub fn cast_vote(ctx: Context<CastVote>, vote_type: u8) -> Result<()> {
        instructions::cast_vote::cast_vote(ctx, vote_type)
    }

    pub fn close_proposal(ctx: Context<CloseProposal>) -> Result<()> {
        instructions::close_proposal::close_proposal(ctx)
    }
}