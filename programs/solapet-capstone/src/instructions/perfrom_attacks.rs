use anchor_lang::prelude::*;

use crate::{error::ErrorCode, verify_ed25519_signature, PetDuel};

#[derive(Accounts)]
pub struct PetAttack<'info> {
    #[account(mut)]
    pub attacker: Signer<'info>,

    /// CHECK: ?
    pub challanger: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"pet_duel", challanger.key().as_ref()],
        bump = pet_duel_account.bump
    )]
    pub pet_duel_account: Account<'info, PetDuel>,

    pub system_program: Program<'info, System>,
    /// CHECK: This is the instructions sysvar account
    pub instructions_sysvar: AccountInfo<'info>,
}

impl<'info> PetAttack<'info> {
    pub fn attack(&mut self, sig: &[u8]) -> Result<()> {
        if self.pet_duel_account.challenger_turn {
            require_keys_eq!(
                self.pet_duel_account.challenger,
                self.attacker.key(),
                ErrorCode::NotChallengerTurn
            );
        } else {
            require_keys_eq!(
                self.pet_duel_account.defender,
                self.attacker.key(),
                ErrorCode::NotDefenderTurn
            );
        }

        self.pet_duel_account.perform_attack(&sig)?;

        self.pet_duel_account.next_turn()?;

        Ok(())
    }
}
