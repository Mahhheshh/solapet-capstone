use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::PetDuel;

#[derive(Accounts)]
pub struct ClaimBetAmount<'info> {
    #[account(mut)]
    pub winner: Signer<'info>,

    /// CHECK: ?
    pub challanger: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"pet_duel", challanger.key().as_ref()],
        bump = pet_duel_account.bump,
    )]
    pub pet_duel_account: Account<'info, PetDuel>,

    #[account(
        seeds = [b"vault"],
        bump
    )]
    pub game_vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> ClaimBetAmount<'info> {
    pub fn claim(&mut self, bumps: &ClaimBetAmountBumps) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.game_vault.to_account_info(),
            to: self.winner.to_account_info(),
        };

        let signer_seeds: &[&[&[u8]]] = &[&[b"vault", &[bumps.game_vault]]];

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer(cpi_context, self.pet_duel_account.bet_amount)?;

        Ok(())
    }
}
