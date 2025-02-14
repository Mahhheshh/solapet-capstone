use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked}};

use crate::{GameConfig, PetStats};

#[derive(Accounts)]
pub struct DepositNft<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    pub collection_mint: Account<'info, Mint>,

    pub player_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"config"],
        bump = config.bump,
        constraint = config.collection_mint.as_ref() == collection_mint.key().as_ref()
    )]
    pub config: Account<'info, GameConfig>,

    #[account(
        init, 
        payer = player, 
        associated_token::mint = collection_mint,
        associated_token::authority = config
    )]
    pub game_ata: Account<'info, TokenAccount>,

    #[account(
        init,
        space = PetStats::INIT_SPACE,
        payer = player, 
        seeds = [b"stats", player.key().as_ref()],
        bump
    )]
    pub pet_stats: Account<'info, PetStats>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
}

impl<'info> DepositNft<'info> {

    pub fn deposit_nft(&mut self) -> Result<()> { 
        let cpi_accounts = TransferChecked {
            from: self.player_ata.to_account_info(),
            to: self.game_ata.to_account_info(),
            authority: self.player.to_account_info(),
            mint: self.collection_mint.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(cpi_ctx, 1, self.collection_mint.decimals)?;

        Ok(())
    }

    pub fn init_pet_stats(&mut self, bumps: &DepositNftBumps) -> Result<()> { 
        let now = Clock::get()?.unix_timestamp;

        self.pet_stats.set_inner(PetStats {
            hunger: 100,
            hygiene: 100,
            energy: 100,
            last_fed_timestamp: now,
            last_bathed_timestamp: now,
            last_slept_timestamp: now,
            bump: bumps.pet_stats,
            // nft_mint: self.collection_mint.key()
        });

        Ok(())
    }
}
