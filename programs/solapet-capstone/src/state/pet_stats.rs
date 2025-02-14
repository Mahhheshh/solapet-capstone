use anchor_lang::prelude::*;

use crate::ANCHOR_DISCRIMINATOR;

#[account]
pub struct PetStats {
    pub hunger: u8,
    pub hygiene: u8,
    pub energy: u8,

    pub last_fed_timestamp: i64,
    pub last_bathed_timestamp: i64,
    pub last_slept_timestamp: i64,

    // pub nft_mint: Pubkey,
    pub bump: u8,
}

impl PetStats {
    pub const INIT_SPACE: usize = ANCHOR_DISCRIMINATOR + 1 + 1 + 1 + 8 + 8 + 8 + 1;

    pub fn feed(&mut self) -> Result<()> {
        self.hunger = 100;
        self.last_fed_timestamp = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn bath(&mut self) -> Result<()> {
        self.hygiene = 100;
        self.last_bathed_timestamp = Clock::get()?.unix_timestamp;
        Ok(())
    }
}
