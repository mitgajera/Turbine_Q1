use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_pack::Pack;
use anchor_spl::token::{spl_token, Token};
use crate::state::{Proposal, Vote};

#[derive(Accounts)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    #[account(
        
        init, payer = voter, space = 8 + Vote::INIT_SPACE, seeds = [b"vote", voter.key().as_ref(),proposal.key().as_ref()], bump
    
    )]

    pub vote_account: Account<'info, Vote>,
    /// CHECK: validated against the token program and voter in handler.
    pub creator_token_account: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn cast_vote(ctx: Context<CastVote>, vote_type: u8) -> Result<()> {
    let vote_account = &mut ctx.accounts.vote_account;
    let proposal_account = &mut ctx.accounts.proposal;
    let creator_token_account = ctx.accounts.creator_token_account.to_account_info();
    if creator_token_account.owner != ctx.accounts.token_program.key {
        return Err(anchor_lang::error::ErrorCode::AccountOwnedByWrongProgram.into());
    }

    let creator_token_account_data = creator_token_account.try_borrow_data()?;
    let creator_token_state = spl_token::state::Account::unpack(&creator_token_account_data)?;

    if creator_token_state.owner != ctx.accounts.voter.key() {
        return Err(anchor_lang::error::ErrorCode::ConstraintOwner.into());
    }

    let voting_credits = (creator_token_state.amount as f64).sqrt() as u64;

    vote_account.set_inner(Vote {
        vote_type,
        authority: ctx.accounts.voter.key(),
        vote_credits: voting_credits,
        bump: ctx.bumps.vote_account,
    });

    match vote_type {
        0 => proposal_account.no_vote_count += voting_credits,
        1 => proposal_account.yes_vote_count += voting_credits,
        _ => return Err(anchor_lang::error::ErrorCode::InstructionFallbackNotFound.into()),
    }

    Ok(())
}