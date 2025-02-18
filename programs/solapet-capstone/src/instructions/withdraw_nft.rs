use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked}};

use crate::{GameConfig, PetStats};

#[derive(Accounts)]
pub struct WithdrawNFT<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    pub collection_mint: Account<'info, Mint>,

    pub nft_mint: Account<'info, Mint>, 

    #[account(mut)]
    pub player_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"game_config"],
        bump = config.bump,
        constraint = config.collection_mint.as_ref() == collection_mint.key().as_ref()
    )]
    pub config: Account<'info, GameConfig>,

    #[account(
        mut, 
        close = player,
        associated_token::mint = nft_mint,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub game_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        close = player,
        seeds = [b"stats", player.key().as_ref()],
        bump
    )]
    pub pet_stats: Account<'info, PetStats>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
}

impl<'info> WithdrawNFT<'info> {
    pub fn withdraw(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked {
            from: self.game_ata.to_account_info(),
            mint: self.nft_mint.to_account_info(),
            to: self.player_ata.to_account_info(),
            authority: self.config.to_account_info(),
        };

        let signer_seeds: &[&[&[u8]]] = &[
            &[
                b"game_config",
                &[self.config.bump]
            ]
        ];


        let cpi_ctx = CpiContext::new_with_signer(
            cpi_program,
            cpi_accounts,
            signer_seeds
        );

        transfer_checked(cpi_ctx, 1, self.nft_mint.decimals)?;
        Ok(())
    }
}