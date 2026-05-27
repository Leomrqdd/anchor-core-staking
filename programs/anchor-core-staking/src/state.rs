use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]

pub struct Config {
    pub rewards_bps: u16,
    pub freeze_period: u16,
    pub reward_bump: u8, //reward mint bump
    pub bump: u8, // config account bump
}