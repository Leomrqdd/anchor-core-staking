use anchor_lang::prelude::*;
use crate::{error::ErrorCode, state::Config};

use mpl_core::{
    ID as MPL_CORE_ID,
    accounts::BaseCollectionV1,
};
use anchor_spl::token_interface::{Mint,TokenInterface};

#[derive(Accounts)]

pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = Config::DISCRIMINATOR.len() + Config::INIT_SPACE,
        seeds = [b"config", collection.key().as_ref()],
        bump,
    )]
    pub config: Account<'info, Config>,
    #[account(has_one = update_authority @ErrorCode::InvalidUpdateAuthority)]
    pub collection: Account<'info, BaseCollectionV1>,
    /// CHECK: this account is not initialized and is being used for signing purpose only, we verify that it derives from the correct seeds
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump
    )]
    pub update_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = admin,
        mint::decimals = 6,
        mint::authority = config,
        seeds = [b"reward_mint", config.key().as_ref()],
        bump,
    )]
    pub rewards_mint: InterfaceAccount<'info,Mint>,
    pub system_program: Program<'info, System>,
    /// CHECK: this is this ID of the MPL Core Program and we check it
    #[account(address = MPL_CORE_ID)]
    pub token_program:Interface<'info,TokenInterface>,
}


impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, rewards_bps: u16, freeze_period: u16, config_bump: u8, reward_bump: u8) -> Result<()> {
        self.config.set_inner(Config {
            rewards_bps,
            freeze_period,
            reward_bump,
            bump: config_bump,
        });

        Ok(())
    }
}

