use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{error::ErrorCode, GameConfig, PetDuel, PetStats};

#[derive(Accounts)]
pub struct AcceptPetDuel<'info> {
    #[account(mut)]
    pub defender: Signer<'info>,

    /// CHECK: ?
    pub challenger: AccountInfo<'info>,

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
        seeds = [b"stats", defender.key().as_ref()],
        bump = pet_stats.bump
    )]
    pub pet_stats: Account<'info, PetStats>,

    #[account(
        mut,
        seeds = [b"pet_duel", challenger.key().as_ref()],
        bump = pet_duel_account.bump
    )]
    pub pet_duel_account: Account<'info, PetDuel>,

    pub system_program: Program<'info, System>,
}

impl<'info> AcceptPetDuel<'info> {
    pub fn accept_duel(&mut self) -> Result<()> {
        let updated_pet_energy = self.pet_stats.update_pet_energy()?;
        require!(updated_pet_energy >= 20, ErrorCode::InsufficientPetEnergy);
        self.pet_duel_account.accept_duel(self.defender.key())?;
        Ok(())
    }

    pub fn deposite(&mut self) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.defender.to_account_info(),
            to: self.game_vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, self.pet_duel_account.bet_amount)?;

        Ok(())
    }
}
