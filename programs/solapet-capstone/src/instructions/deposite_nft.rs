use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use mpl_token_metadata::instructions::{
    DelegateStandardV1Cpi, DelegateStandardV1CpiAccounts, DelegateStandardV1InstructionArgs, LockV1Cpi, LockV1CpiAccounts, LockV1InstructionArgs
};

use crate::{GameConfig, PetStats};
#[derive(Accounts)]
pub struct DepositNft<'info> {
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

    /// The Metaplex token metadata program
    /// CHECK: This is the Metaplex Token Metadata Program
    #[account(address = mpl_token_metadata::ID)]
    pub token_metadata_program: UncheckedAccount<'info>,

    /// CHECK: Address is validated to be the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::id())]
    pub sysvar_instructions: UncheckedAccount<'info>,

    #[account(
        init,
        space = PetStats::INIT_SPACE,
        payer = player, 
        seeds = [b"stats", player.key().as_ref()],
        bump
    )]
    pub pet_stats: Account<'info, PetStats>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> DepositNft<'info> {
    pub fn freeze_nft(&mut self) -> Result<()> {    
        let master_edition_info = &self.master_edition.to_account_info();
        let player_info = &self.player.to_account_info();
        let token_program_info = &self.token_program.to_account_info();
        let player_ata_info = &self.player_ata.to_account_info();
        
        let signers_seeds: &[&[&[u8]]] = &[&[b"game_config", &[self.config.bump]]];

        let cpi_accounts = DelegateStandardV1CpiAccounts {
            delegate_record: None, 
            delegate: &self.config.to_account_info(), // give access to the config
            metadata: &self.metadata.to_account_info(), 
            master_edition: Some(master_edition_info), 
            token_record: None, 
            mint: &self.nft_mint.to_account_info(), 
            token: player_ata_info, 
            authority: player_info, 
            payer: player_info, 
            system_program: &self.system_program.to_account_info(),
            sysvar_instructions: &self.sysvar_instructions.to_account_info(), 
            spl_token_program: Some(token_program_info), 
            authorization_rules_program: None,
            authorization_rules: None, 
        };

        let cpi_args = DelegateStandardV1InstructionArgs {
            amount: 1,
        };

        DelegateStandardV1Cpi::new(
            &self.token_metadata_program.to_account_info(),
            cpi_accounts,
            cpi_args,
        ).invoke_signed(&signers_seeds)?;
        
        let cpi_accounts = LockV1CpiAccounts {
            mint: &self.nft_mint.to_account_info(),
            authority: &self.config.to_account_info(),
            payer: player_info, 
            system_program: &self.system_program.to_account_info(),
            token_owner: Some(player_info),
            token: player_ata_info, 
            metadata: &self.metadata.to_account_info(),
            edition: Some(master_edition_info),
            token_record: None,
            sysvar_instructions: &self.sysvar_instructions.to_account_info(),
            spl_token_program: Some(token_program_info), 
            authorization_rules_program: None,
            authorization_rules: None,
        };

        let cpi_args = LockV1InstructionArgs {
            authorization_data: None,
        };

        LockV1Cpi::new(
            &self.token_metadata_program.to_account_info(),
            cpi_accounts,
            cpi_args
        ).invoke_signed(&signers_seeds)?;
        
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
