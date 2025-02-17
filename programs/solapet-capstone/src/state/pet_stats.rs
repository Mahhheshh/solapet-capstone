use anchor_lang::prelude::*;

use crate::{error::ErrorCode, ANCHOR_DISCRIMINATOR};

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

    pub fn sleep(&mut self) -> Result<()> {
        let current_timestamp = Clock::get()?.unix_timestamp;

        let sleep_delta = ((current_timestamp - self.last_slept_timestamp) / 900) as u8;

        require!(sleep_delta >= 30, ErrorCode::InsufficientPetEnergy);
        self.energy = 100; // TODO: Consider implementing gradual energy restoration
        self.last_slept_timestamp = current_timestamp;

        Ok(())
    }

    pub fn update_pet_energy(&mut self) -> Result<u8> {
        let current_timestamp = Clock::get()?.unix_timestamp;

        let energy_delta = ((current_timestamp - self.last_slept_timestamp) / 3600) as u8;
        self.energy = self.energy.saturating_sub(energy_delta).clamp(0, 100);
        Ok(self.energy)
    }

    pub fn update_pet_hygiene(&mut self) -> Result<u8> {
        let current_timestamp = Clock::get()?.unix_timestamp;

        let hygiene_delta = ((current_timestamp - self.last_bathed_timestamp) / 3600) as u8;
        self.hygiene = self.hygiene.saturating_sub(hygiene_delta).clamp(0, 100);
        Ok(self.hygiene)
    }

    pub fn update_pet_hunger(&mut self) -> Result<u8> {
        let current_timestamp = Clock::get()?.unix_timestamp;

        let hunger_delta = ((current_timestamp - self.last_fed_timestamp) / 1800) as u8;
        self.hunger = self.hunger.saturating_sub(hunger_delta).clamp(0, 100);
        Ok(self.hunger)
    }

    pub fn update_pet_stats(&mut self) -> Result<()> {
        self.update_pet_energy()?;
        self.update_pet_hygiene()?;
        self.update_pet_hunger()?;
        Ok(())
    }
}
