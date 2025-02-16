use anchor_lang::prelude::*;

use crate::ANCHOR_DISCRIMINATOR;

#[account]
pub struct GameConfig {
    pub admin: Pubkey,
    pub collection_mint: Pubkey,
    pub game_vault: Pubkey,
    pub fees: u8,
    pub bump: u8,
    pub vault_bump: u8,
}

impl GameConfig {
    pub const INIT_SPACE: usize = ANCHOR_DISCRIMINATOR + 32 + 32 + 32 + 1 + 1 + 1;

    pub fn update_fees(&mut self, updated_fees: u8) -> Result<()> {
        self.fees = updated_fees;
        Ok(())
    }
}
