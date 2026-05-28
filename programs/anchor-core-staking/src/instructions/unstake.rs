use anchor_lang::prelude::*;
use mpl_core::{
    ID as MPL_CORE_ID,
    accounts::{BaseAssetV1, BaseCollectionV1},
    types::{UpdateAuthority,Attribute,Attributes,Plugin,PluginType,FreezeDelegate},
    instructions::{UpdateCollectionPluginV1CpiBuilder, UpdatePluginV1CpiBuilder},
    fetch_plugin,
};
use crate::{error::ErrorCode, state::Config};

const SECONDS_PER_DAY: i64 = 86400;

#[derive(Accounts)]

pub struct Unstake<'info> {
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


impl<'info> Unstake<'info> {
    pub fn unstake(&self, bump: u8) -> Result<()> {
        let attributes_fetched = fetch_plugin::<BaseAssetV1,Attributes>(
            &self.asset.to_account_info(),
            PluginType::Attributes,
        )
        .ok()
        .map(|(_,attrs,_)| attrs);

        require!(attributes_fetched.is_some(), ErrorCode::AssetNotStaked);
    
        let attributes = attributes_fetched.unwrap();

        let mut attributes_list:Vec<Attribute> = Vec::with_capacity(attributes.attribute_list.len());

        let current_timestamp = Clock::get()?.unix_timestamp;
        let mut staked_timestamp: i64 = 0;
        let mut staked_time: i64 = 0;

        for attribute in &attributes.attribute_list {
            if attribute.key == "staked" {
                require!(attribute.value == "true", ErrorCode::AssetNotStaked);
            }
            else if attribute.key == "staked_at" {
                staked_timestamp = attribute.value.parse::<i64>().map_err(|_| ErrorCode::InvalidTimestamp)?; 
                staked_time = current_timestamp.checked_sub(staked_timestamp).ok_or(ErrorCode::InvalidTimestamp)?;
                staked_time = staked_time.checked_div(SECONDS_PER_DAY).ok_or(ErrorCode::InvalidTimestamp)?;
                require!(staked_time >= self.config.freeze_period as i64, ErrorCode::FreezePeriodNotElapsed);
            }
            else {
                attributes_list.push(attribute.clone());
            }
        }

        attributes_list.push(Attribute {
            key: "staked".to_string(),
            value: "false".to_string(),
        });

        attributes_list.push(Attribute {
            key: "staked_at".to_string(),
            value: "0".to_string(),
        });


       let collection_key = self.collection.key();
       let signer_seeds: &[&[&[u8]]] = &[&[
            b"update_authority",
            collection_key.as_ref(),
            &[bump],
        ]];

        UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .asset(&self.asset.to_account_info())
            .payer(&self.owner.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::Attributes(Attributes { attribute_list: attributes_list }))
            .invoke_signed(signer_seeds)?;

        UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .asset(&self.asset.to_account_info())
            .payer(&self.owner.to_account_info())
            .authority(Some(&self.owner.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::FreezeDelegate(FreezeDelegate {frozen: false }))
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
            value: staked_count.checked_sub(1).ok_or(ErrorCode::InvalidStakeCount)?.to_string(),
        });

        UpdateCollectionPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .collection(&self.collection.to_account_info())
            .payer(&self.owner.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::Attributes(Attributes { attribute_list: collection_attr_list }))
            .invoke_signed(signer_seeds)?;

        Ok(())
    }
}

