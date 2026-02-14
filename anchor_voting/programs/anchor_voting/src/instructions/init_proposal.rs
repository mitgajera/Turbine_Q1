use anchor_lang::prelude::*;
use crate::state::{Dao, Proposal};

#[derive(Accounts)]
#[instruction(metadata: String)]
pub struct InitProposal<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"dao", creator.key().as_ref(), dao_account.name.as_bytes()],
        bump = dao_account.bump
    )]
    pub dao_account: Account<'info, Dao>,

    #[account(
        init,
        payer = creator,
        space = 8 + Proposal::INIT_SPACE,
        seeds = [
            b"proposal",
            dao_account.key().as_ref(),
            &dao_account.proposal_account.to_le_bytes()
        ],
        bump
    )]
    pub proposal: Account<'info, Proposal>,

    pub system_program: Program<'info, System>,
}

pub fn init_proposal(ctx: Context<InitProposal>, metadata: String) -> Result<()> {
    let dao_account = &mut ctx.accounts.dao_account;
    let proposal = &mut ctx.accounts.proposal;

    proposal.set_inner(Proposal {
        metadata,
        authority: ctx.accounts.creator.key(),
        yes_vote_count: 0,
        no_vote_count: 0,
        bump: ctx.bumps.proposal,
    });

    dao_account.proposal_account += 1;
    Ok(())
}