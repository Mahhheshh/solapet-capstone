use anchor_lang::prelude::*;

use crate::{error::ErrorCode, GameConfig};

#[derive(Accounts)]
pub struct UpdateFees<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"game_config"],
        bump = game_config.bump,
        has_one = admin @ ErrorCode::UnauthorizedAction
    )]
    pub game_config: Account<'info, GameConfig>,
}

impl<'info> UpdateFees<'info> {
    pub fn update_fees(&mut self, new_fees: u8) -> Result<()> {
        self.game_config.update_fees(new_fees)?;
        Ok(())
    }
}
