use anchor_lang::prelude::*;
use mpl_core::{
    instructions::UpdatePluginV1CpiBuilder,
    types::{FreezeDelegate, Plugin},
    ID as CORE_PROGRAM_ID,
};

use crate::{error::MPLXCoreError, state::CollectionAuthority};

#[derive(Accounts)]
pub struct FreezeNft<'info> {
    #[account(
        mut,
        constraint = collection_authority.creator == authority.key() @MPLXCoreError::UnauthorizedCreator,
    )]

    pub authority: Signer<'info>,

    #[account(mut)]
    pub asset:: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = constraint.owner == &CORE_PROGRAM_ID @MPLXCoreError::InvalidCollection,
        constraint = !collection.data_is_empty() @MPLXCoreError::CollectionNotInitialized,
    )]

    pub collection: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [
            b"collection_authority".as_ref(),
            collection.key().as_ref(),
        ],
        bump = collection_authority.bump,
    )]

    pub collection_authority: Account<'info, CollectionAuthority>,

    #[account(address = CORE_PROGRAM_ID)]
    pub core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>
}

impl<'info> FreezeNft<'info> {
    pub fn freeze_nft(&mut self) -> Result<()> {

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"collection_authority".as_ref(),
            self.collection.key().as_ref(),
            &[self.collection_authority.bump],
        ]];

        UpdatePluginV1CpiBuilder::new(&self.core_program)
        .asset(&self.asset)
        .collection(&self.collection)
        .payer(&self.authority.to_account_info())
        .authority(&self.collection_authority.to_account_info())
        .system_program(&self.system_program.to_account_info())
        .plugin(Plugin::FreezeDelegate(FreezeDelegate {frozen: true}))
        .invoke_signed(signer_seeds)?;

        Ok(())
    }
}
