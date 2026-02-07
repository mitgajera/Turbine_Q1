use anchor_lang::{prelude::*, solana_program::bpf_loader_upgradeable};

use crate::{error::MPLXCoreError, state::WhitelistedCreators};

#[derive(Accounts)] 
pub struct WhitelistCreator<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK should be a keypair
    pub creator: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = payer,
        space = WhitelistedCreators::DISCRIMINATOR.len() + WhitelistedCreators::INIT_SPACE,
        seeds = [b"whitelist"],
        bump,
    )]
    pub whitelisted_creators: Account<'info, WhitelistedCreators>,
    pub system_program: Program<'info, System>,
    #[account(address = crate::ID)]
    /// CHECK: this is the current program id
    pub this_program: UncheckedAccount<'info>,
    // Making sure only the program update authority can add creators to the array
    #[account(
        constraint = program_data.key() == Pubkey::find_program_address(
            &[this_program.key().as_ref()],
            &bpf_loader_upgradeable::ID
        ).0,
        constraint = program_data.upgrade_authority_address == Some(payer.key()) @ MPLXCoreError::UnauthorizedCreator,
    )]
    pub program_data: Account<'info, ProgramData>,
}

impl<'info> WhitelistCreator<'info> {
    pub fn whitelist_creator(&mut self) -> Result<()> {
        self.whitelisted_creators.whitelist_creator(&self.creator)
    }
}