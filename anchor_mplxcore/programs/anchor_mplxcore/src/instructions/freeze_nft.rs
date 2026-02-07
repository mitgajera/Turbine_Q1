use anchor_lang::prelude::*;
use mpl_core::{
    instructions::UpdatePluginV1CpiBuilder,
    types::{FreezeDelegate, Plugin},
    ID as CORE_PROGRAM_ID,
};

use crate::{error::MPLXCoreError, state::CollectionAuthority};

#[derive(Accounts)]
pub struct FreezeNft<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    /// CHECK: asset is validated by the core program during CPI
    pub asset: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = collection.owner == &CORE_PROGRAM_ID @MPLXCoreError::InvalidCollection,
        constraint = !collection.data_is_empty() @MPLXCoreError::CollectionNotInitialized,
    )]
    /// CHECK: collection is validated by the core program during CPI
    pub collection: UncheckedAccount<'info>,
    pub collection_authority: Account<'info, CollectionAuthority>,

    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: core program id is verified by address constraint
    pub core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>
}

impl<'info> FreezeNft<'info> {
    pub fn freeze_nft(&mut self) -> Result<()> {
        let (expected_pda, _) = Pubkey::find_program_address(
            &[b"collection_authority", self.collection.key().as_ref()],
            &crate::ID,
        );
        require_keys_eq!(
            expected_pda,
            self.collection_authority.key(),
            MPLXCoreError::InvalidCollection
        );
        let collection_authority = &self.collection_authority;
        require_keys_eq!(
            collection_authority.creator,
            self.authority.key(),
            MPLXCoreError::NotAuthorized
        );

        let binding = self.collection.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"collection_authority".as_ref(),
            binding.as_ref(),
            &[collection_authority.bump],
        ]];

        UpdatePluginV1CpiBuilder::new(&self.core_program)
        .asset(&self.asset)
        .collection(Some(&self.collection))
        .payer(&self.authority.to_account_info())
        .authority(Some(&collection_authority.to_account_info()))
        .system_program(&self.system_program.to_account_info())
        .plugin(Plugin::FreezeDelegate(FreezeDelegate {frozen: true}))
        .invoke_signed(signer_seeds)?;

        Ok(())
    }
}
