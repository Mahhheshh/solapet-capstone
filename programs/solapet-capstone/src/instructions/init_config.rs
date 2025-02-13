use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::GameConfig;

#[derive(Accounts)]
pub struct InitializeGameConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    pub collection_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        space = GameConfig::INIT_SPACE,
        seeds = [b"game_config"],
        bump
    )]
    pub game_config: Account<'info, GameConfig>,

    #[account(
        seeds = [b"vault"],
        bump
    )]
    pub game_vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitializeGameConfig<'info> {
    pub fn init_game_config(&mut self, bumps: &InitializeGameConfigBumps, fees: u8) -> Result<()> {
        self.game_config.set_inner(GameConfig {
            admin: self.admin.key(),
            collection_mint: self.collection_mint.key(),
            game_vault: self.game_vault.key(),
            fees,
            bump: bumps.game_config,
            vault_bump: bumps.game_vault,
        });

        Ok(())
    }
}
