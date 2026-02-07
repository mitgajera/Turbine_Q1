use anchor_lang::prelude::*;
use mpl_core::{
    instructions::UpdateV2CpiBuilder,
    ID as CORE_PROGRAM_ID,
};

use crate::{error::MPLXCoreError, state::CollectionAuthority};

#[derive(Accounts)]
pub struct UpdateNft<'info> {
    #[account(
        mut,
        constraint = collection_authority.creator == authority.key() @MPLXCoreError::UnauthorizedCreator,
    )]
    pub authority: Signer<'info>,

    /// CHECK: Checked by metaplex core
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = collection.owner == &CORE_PROGRAM_ID @MPLXCoreError::InvalidCollection,
        constraint = !collection.data_is_empty() @MPLXCoreError::CollectionNotInitialized,
    )]
    /// CHECK: verified by metaplex core
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
    /// CHECK: verified by metaplex core
    pub core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> UpdateNft<'info> {
    pub fn update_nft(&mut self, name: String, uri: String) -> Result<()> {
        let binding = self.collection.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"collection_authority".as_ref(),
            binding.as_ref(),
            &[self.collection_authority.bump],
        ]];

        UpdateV2CpiBuilder::new(&self.core_program)
            .asset(&self.asset)
            .collection(Some(&self.collection))
            .payer(&self.authority.to_account_info())
            .authority(Some(&self.collection_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .new_name(name)
            .new_uri(uri)
            .invoke_signed(signer_seeds)?;

        Ok(())
    }
}