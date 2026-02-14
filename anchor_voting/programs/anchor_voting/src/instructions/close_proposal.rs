use anchor_lang::prelude::*;
use crate::state::Proposal;

#[derive(Accounts)]
pub struct CloseProposal<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    pub system_program: Program<'info, System>,
}

pub fn close_proposal(ctx: Context<CloseProposal>) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    proposal.set_inner(Proposal {
        metadata: proposal.metadata.clone(),
        authority: ctx.accounts.authority.key(),
        yes_vote_count: proposal.yes_vote_count,
        no_vote_count: proposal.no_vote_count,
        bump: proposal.bump,
    });
    Ok(())
}

