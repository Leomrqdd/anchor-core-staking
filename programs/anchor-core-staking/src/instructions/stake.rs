use anchor_lang::prelude::*;
use mpl_core::{
    ID as MPL_CORE_ID, accounts::{BaseAssetV1, BaseCollectionV1}, fetch_plugin, instructions::{AddPluginV1CpiBuilder, UpdateCollectionPluginV1CpiBuilder, UpdatePluginV1, UpdatePluginV1CpiBuilder}, types::{Attribute, Attributes, FreezeDelegate, Plugin, PluginAuthority, PluginType, UpdateAuthority}
};

use crate::{error::ErrorCode, state::Config};

#[derive(Accounts)]

pub struct Stake<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        seeds = [b"config", collection.key().as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        has_one = owner @ErrorCode::InvalidOwner,
        constraint = asset.update_authority == UpdateAuthority::Collection(collection.key()) @ErrorCode::InvalidUpdateAuthority,
    )]
    pub asset: Account<'info, BaseAssetV1>,
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


impl<'info> Stake<'info> {
    pub fn stake(&self, bump: u8) -> Result<()> {
        let attributes_fetched = fetch_plugin::<BaseAssetV1,Attributes>(
            &self.asset.to_account_info(),
            PluginType::Attributes,
        )
        .ok()
        .map(|(_,attrs,_)| attrs);

        let mut attributes_list = Vec::new();

        if let Some(attributes) = &attributes_fetched {
            for attribute in &attributes.attribute_list {
                if attribute.key == "staked" {
                    require!(attribute.value == "false", ErrorCode::AlreadyStaked);
                }
                else if attribute.key != "staked_at" {
                    attributes_list.push(attribute.clone());
                }
            }
        }

        attributes_list.push(Attribute {
            key: "staked".to_string(),
            value: "true".to_string(),
        });

        attributes_list.push(Attribute {
            key: "staked_at".to_string(),
            value: Clock::get()?.unix_timestamp.to_string(),
        });

        let collection_key = self.collection.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"update_authority",
            collection_key.as_ref(),
            &[bump],
        ]];

        if attributes_fetched.is_none() {
            AddPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                .collection(Some(&self.collection.to_account_info()))
                .asset(&self.asset.to_account_info())
                .payer(&self.owner.to_account_info())
                .authority(Some(&self.update_authority.to_account_info()))
                .system_program(&self.system_program.to_account_info())
                .plugin(Plugin::Attributes(Attributes { attribute_list: attributes_list }))
                .invoke_signed(signer_seeds)?;
        }
        else {
            UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                .collection(Some(&self.collection.to_account_info()))
                .asset(&self.asset.to_account_info())
                .authority(Some(&self.update_authority.to_account_info()))
                .system_program(&self.system_program.to_account_info())
                .plugin(Plugin::Attributes(Attributes { attribute_list: attributes_list }))
                .invoke_signed(signer_seeds)?;
        }

        AddPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .asset(&self.asset.to_account_info())
            .payer(&self.owner.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::FreezeDelegate(FreezeDelegate {
                frozen: true,
            }))
            .invoke()?;

    
        let collection_info = self.collection.to_account_info();
        let (_, collection_attrs, _) = fetch_plugin::<BaseCollectionV1, Attributes>(
            &collection_info,
            PluginType::Attributes,
        )?;

        let mut staked_count: u64 = 0;
        let mut collection_attr_list: Vec<Attribute> = collection_attrs.attribute_list
            .into_iter()
            .filter(|attr| {
                if attr.key == "staked_count" {
                    staked_count = attr.value.parse().unwrap_or(0);
                    false
                } else {
                    true
                }
            })
            .collect();

        collection_attr_list.push(Attribute {
            key: "staked_count".to_string(),
            value: staked_count.checked_add(1).ok_or(ErrorCode::InvalidStakeCount)?.to_string(),
        });

        UpdateCollectionPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .collection(&self.collection.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::Attributes(Attributes { attribute_list: collection_attr_list }))
            .invoke_signed(signer_seeds)?;

        Ok(())
    }
}

