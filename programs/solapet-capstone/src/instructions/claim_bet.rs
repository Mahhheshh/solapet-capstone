use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{error::ErrorCode, GameConfig, PetDuel};

#[derive(Accounts)]
pub struct ClaimBetAmount<'info> {
    #[account(mut)]
    pub winner: Signer<'info>,

    /// CHECK: ?
    pub challanger: AccountInfo<'info>,

    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump
    )]
    pub game_config: Account<'info, GameConfig>,

    #[account(
        mut,
        seeds = [b"pet_duel", challanger.key().as_ref()],
        bump = pet_duel_account.bump,
        close = winner
    )]
    pub pet_duel_account: Account<'info, PetDuel>,

    #[account(
        mut,
        seeds = [b"vault"],
        bump = game_config.vault_bump
    )]
    pub game_vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> ClaimBetAmount<'info> {
    pub fn claim(&mut self) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.game_vault.to_account_info(),
            to: self.winner.to_account_info(),
        };

        let signer_seeds: &[&[&[u8]]] = &[&[b"vault", &[self.game_config.vault_bump]]];

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        let bet_amount = self.pet_duel_account.bet_amount;

        let deduced_amount = bet_amount
            .checked_mul(self.game_config.fees as u64)
            .unwrap()
            .checked_div(100)
            .unwrap();

        let transferable_amount = bet_amount.saturating_sub(deduced_amount);
        transfer(cpi_context, transferable_amount)?;

        Ok(())
    }
}
