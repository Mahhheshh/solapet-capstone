use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{GameConfig, PetStats};
use mpl_token_metadata::{
    types::DelegateArgs,
    instructions::{
        DelegateCpiAccounts, 
        DelegateInstructionArgs, 
        DelegateCpi,
        LockV1CpiAccounts,
        LockV1InstructionArgs,
        LockV1Cpi
    }
};

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
        let spl_token_program_info = &self.token_program.to_account_info();
        let player_info = &self.player.to_account_info();
        let master_edition = &self.master_edition.to_account_info();
        let player_ata = self.player_ata.to_account_info();

        // First delegate authority to the game config
        let delegate_accounts = DelegateCpiAccounts {
            delegate: &self.config.to_account_info(),
            metadata: &self.metadata,
            master_edition: Some(master_edition),
            token_record: None,
            mint: &self.nft_mint.to_account_info(),
            token: Some(&player_ata), 
            authority: player_info,
            payer: player_info,
            system_program: &self.system_program.to_account_info(),
            sysvar_instructions: &self.sysvar_instructions,
            spl_token_program: Some(spl_token_program_info),
            authorization_rules_program: None,
            authorization_rules: None,
            delegate_record: None,
        };

        let delegate_args = DelegateInstructionArgs {
            delegate_args: DelegateArgs::StakingV1 {
                amount: 1,
                authorization_data: None
            }
        };

        DelegateCpi::new(
            &self.token_metadata_program,
            delegate_accounts,
            delegate_args
        ).invoke()?;

        // Then lock the NFT
        let lock_accounts = LockV1CpiAccounts {
            authority: &self.config.to_account_info(),
            token_owner: Some(player_info),
            token: &self.player_ata.to_account_info(),
            mint: &self.nft_mint.to_account_info(),
            metadata: &self.metadata,
            edition: Some(master_edition),
            token_record: None,
            system_program: &self.system_program.to_account_info(),
            sysvar_instructions: &self.sysvar_instructions,
            spl_token_program: Some(spl_token_program_info),
            authorization_rules_program: None,
            authorization_rules: None,
            payer: player_info,
        };

        let lock_args = LockV1InstructionArgs {
            authorization_data: None,
        };

        let signer_seeds: &[&[&[u8]]] = &[&[b"game_config", &[self.config.bump]]];

        LockV1Cpi::new(
            &self.token_metadata_program,
            lock_accounts,
            lock_args
        ).invoke_signed(signer_seeds)?;
                
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
