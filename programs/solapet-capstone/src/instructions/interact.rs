use anchor_lang::prelude::*;

use crate::{error::ErrorCode, PetStats};

#[derive(Accounts)]
pub struct PetInteract<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
        mut, 
        seeds = [b"stats", player.key().as_ref()], 
        bump = pet_stats.bump
    )]
    pub pet_stats: Account<'info, PetStats>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum InteractionType {
    Feed,
    Bath
}

impl<'info> PetInteract<'info> {
    pub fn interact(&mut self, interaction_type: InteractionType) -> Result<()> {
        match interaction_type {
            InteractionType::Feed => self.pet_stats.feed()?,
            InteractionType::Bath => self.pet_stats.bath()?,
            // _ => return Err(ErrorCode::InvalidPetInteraction)?,
        };
        Ok(())
    }
}