use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use mpl_token_metadata::instructions::{
    RevokeStandardV1Cpi, RevokeStandardV1CpiAccounts, UnlockV1Cpi, UnlockV1CpiAccounts,
    UnlockV1InstructionArgs,
};

use crate::{GameConfig, PetStats};

#[derive(Accounts)]
pub struct WithdrawNFT<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    pub collection_mint: Account<'info, Mint>, // collection_mint

    #[account(mut)]
    pub nft_mint: Account<'info, Mint>, // the nft mint

    #[account(mut)]
    pub player_ata: Account<'info, TokenAccount>, // users token account holding pet nft

    #[account(
        seeds = [b"game_config"],
        bump = config.bump,
        constraint = config.collection_mint.as_ref() == collection_mint.key().as_ref()
    )]
    pub config: Account<'info, GameConfig>,

    /// Master edition account for the NFT
    /// CHECK: Validated by the Metaplex token metadata program
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    /// Metadata account for the NFT
    /// CHECK: Validated by the Metaplex token metadata program
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Address is validated to be the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::id())]
    pub sysvar_instructions: UncheckedAccount<'info>,

    #[account(
        mut,
        close = player,
        seeds = [b"stats", player.key().as_ref()],
        bump
    )]
    pub pet_stats: Account<'info, PetStats>,

    /// The Metaplex token metadata program
    /// CHECK: This is the Metaplex Token Metadata Program
    #[account(address = mpl_token_metadata::ID)]
    pub token_metadata_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> WithdrawNFT<'info> {
    pub fn withdraw(&mut self) -> Result<()> {
        let player_ata = &self.player_ata.to_account_info();
        let master_edition = &self.master_edition.to_account_info();
        let metadata = &self.metadata.to_account_info();
        let sysvar_instructions = &self.sysvar_instructions.to_account_info();
        let token_program = &self.token_program.to_account_info();

        let cpi_accounts = UnlockV1CpiAccounts {
            mint: &self.nft_mint.to_account_info(),
            token: player_ata,
            token_owner: Some(player_ata),
            token_record: None,
            edition: Some(master_edition),
            metadata: metadata,
            authority: &self.config.to_account_info(),
            payer: &self.player.to_account_info(),
            system_program: &self.system_program.to_account_info(),
            sysvar_instructions: sysvar_instructions,
            spl_token_program: Some(token_program),
            authorization_rules_program: None,
            authorization_rules: None,
        };
        let cpi_args = UnlockV1InstructionArgs {
            authorization_data: None,
        };

        let signers_seeds: &[&[&[u8]]] = &[&[b"game_config", &[self.config.bump]]];

        UnlockV1Cpi::new(
            &self.token_metadata_program.to_account_info(),
            cpi_accounts,
            cpi_args,
        )
        .invoke_signed(signers_seeds)?;

        // Revoke the delegate standard v1
        let cpi_accounts = RevokeStandardV1CpiAccounts {
            delegate_record: None,
            delegate: &self.config.to_account_info(),
            metadata: metadata,
            master_edition: Some(master_edition),
            token_record: None,
            mint: &self.nft_mint.to_account_info(),
            token: player_ata,
            authority: &self.player.to_account_info(),
            payer: &self.player.to_account_info(),
            system_program: &self.system_program.to_account_info(),
            sysvar_instructions: sysvar_instructions,
            spl_token_program: Some(token_program),
            authorization_rules_program: None,
            authorization_rules: None,
        };

        RevokeStandardV1Cpi::new(&self.token_metadata_program.to_account_info(), cpi_accounts)
            .invoke_signed(signers_seeds)?;

        Ok(())
    }
}
