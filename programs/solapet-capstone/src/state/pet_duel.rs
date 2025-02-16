// state/pet_duel.rs
use anchor_lang::prelude::*;

use crate::{error::ErrorCode, gen_number, ANCHOR_DISCRIMINATOR};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum DuelStatus {
    Challenged,
    Started,
    Finished,
    InProgress,
}

#[account]
pub struct PetDuel {
    pub challenger: Pubkey,
    pub defender: Pubkey,
    pub winner: Option<Pubkey>,

    pub challenger_pet_health: u8,
    pub defender_pet_health: u8,

    pub bet_amount: u64,

    pub duel_status: DuelStatus,

    pub challenger_turn: bool,

    pub last_turn_timestamp: i64,

    pub bump: u8,
}

impl PetDuel {
    pub const INIT_SPACE: usize = ANCHOR_DISCRIMINATOR + 32 + 32 + 33 + 1 + 1 + 8 + 1 + 1 + 8 + 1;

    pub fn accept_duel(&mut self, defender: Pubkey) -> Result<()> {
        require!(
            self.duel_status == DuelStatus::Challenged,
            ErrorCode::DuelAlreadyStarted
        );
        self.defender = defender;
        self.duel_status = DuelStatus::Started;
        Ok(())
    }

    pub fn next_turn(&mut self) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        self.last_turn_timestamp = now;
        self.challenger_turn = !self.challenger_turn;
        Ok(())
    }

    pub fn perform_attack(&mut self, sig: &[u8]) -> Result<()> {
        let damage: u8 = gen_number(&sig, 40)?;

        if self.challenger_turn {
            self.defender_pet_health = self.defender_pet_health.saturating_sub(damage);
        } else {
            self.challenger_pet_health = self.challenger_pet_health.saturating_sub(damage);
        }

        if self.challenger_pet_health == 0 || self.defender_pet_health == 0 {
            self.duel_status = DuelStatus::Finished;
            if self.challenger_pet_health == 0 {
                self.winner = Some(self.defender);
            } else {
                self.winner = Some(self.challenger);
            }
        }
        Ok(())
    }
}
