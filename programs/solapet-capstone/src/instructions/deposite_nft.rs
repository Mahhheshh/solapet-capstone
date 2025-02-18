use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount, transfer, Transfer}};

use crate::{GameConfig, PetStats};

#[derive(Accounts)]
pub struct DepositNft<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    pub collection_mint: Account<'info, Mint>, // collection_mint

    pub nft_mint: Account<'info, Mint>, // the nft mint

    #[account(mut)]
    pub player_ata: Account<'info, TokenAccount>, // users token account holding pet nft

    #[account(
        seeds = [b"game_config"],
        bump = config.bump,
        constraint = config.collection_mint.as_ref() == collection_mint.key().as_ref()
    )]
    pub config: Account<'info, GameConfig>,

    #[account(
        init, 
        payer = player, 
        associated_token::mint = nft_mint,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub destination: Account<'info, TokenAccount>,

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
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.player_ata.to_account_info(),
            to: self.destination.to_account_info(),
            authority: self.player.to_account_info(),
        };

        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        
        transfer(cpi_context, 1)?;
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
