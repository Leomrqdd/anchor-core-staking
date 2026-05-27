use anchor_lang::prelude::*;
use mpl_core::{
    ID as MPL_CORE_ID,
    instructions::CreateV2CpiBuilder,
    accounts::BaseCollectionV1,
};

use crate::{error::ErrorCode};

#[derive(Accounts)]

pub struct MintAsset<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub asset: Signer<'info>,
    #[account(
        mut,
        has_one = update_authority @ErrorCode::InvalidUpdateAuthority,
    )]
    pub collection: Account<'info, BaseCollectionV1>,
     /// CHECK: this account is not initialized and is being used for signing purpose only, we verify that it derives from the correct seeds
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump
    )]
    pub update_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: this is this ID of the MPL Core Program and we check it
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program:UncheckedAccount<'info>,
}


impl<'info> MintAsset<'info> {
    pub fn mint_asset(&self, name: String, uri: String, bump: u8) -> Result<()> {
        let collection_key = self.collection.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"update_authority",
            collection_key.as_ref(),
            &[bump],
        ]];

        CreateV2CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .owner(Some(&self.user.to_account_info()))
            .update_authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .name(name)
            .uri(uri)
            .invoke_signed(signer_seeds)?;

        Ok(())
    }
}

