use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{DuelStatus, GameConfig, PetDuel};

#[derive(Accounts)]
pub struct InitPetDuel<'info> {
    #[account(mut)]
    pub challanger: Signer<'info>,

    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump,
        has_one = game_vault
    )]
    pub game_config: Account<'info, GameConfig>,

    #[account(
        seeds = [b"vault"],
        bump
    )]
    pub game_vault: SystemAccount<'info>,
    
    #[account(
        init,
        payer = challanger,
        space = PetDuel::INIT_SPACE,
        seeds = [b"pet_duel", challanger.key().as_ref()],
        bump
    )]
    pub pet_duel_account: Account<'info, PetDuel>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitPetDuel<'info> {
    pub fn initilize(&mut self, bumps: &InitPetDuelBumps, bet_amount: u64) -> Result<()> {
        self.pet_duel_account.set_inner(PetDuel {
            challenger: self.challanger.key(),
            defender: Pubkey::default().key(),
            winner: Option::None,
            challenger_pet_health: 100,
            defender_pet_health: 100,
            bet_amount,
            duel_status: DuelStatus::Challenged,
            challenger_turn: true,
            last_turn_timestamp: Clock::get()?.unix_timestamp,
            bump: bumps.pet_duel_account,
        });
        Ok(())
    }

    pub fn deposite(&mut self, bet_amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.challanger.to_account_info(),
            to: self.game_vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, bet_amount)?;

        Ok(())
    }
}
