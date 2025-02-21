use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token},
};
use mpl_token_metadata::{
    instructions::{
        CreateV1Cpi, CreateV1CpiAccounts, CreateV1InstructionArgs, MintV1Cpi, MintV1CpiAccounts,
        MintV1InstructionArgs,
    },
    types::{Collection, PrintSupply, TokenStandard::NonFungible},
};

use crate::GameConfig;

#[derive(Accounts)]
pub struct MintPetNft<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    pub collection_mint: Account<'info, Mint>,

    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump,
        constraint = game_config.collection_mint.as_ref() == collection_mint.key().as_ref() // Verify collection mint matches config
    )]
    pub game_config: Account<'info, GameConfig>,

    /// The NFT mint account that will be initialized
    #[account(
        init,
        payer = player,
        seeds = [b"nft_mint", player.key().as_ref(), collection_mint.key().as_ref()],
        bump,
        mint::decimals = 0,
        mint::authority = game_config,
        mint::freeze_authority = game_config
    )]
    pub nft_mint: Account<'info, Mint>,

    /// CHECK: This is the associated token account that will be created
    #[account(mut)]
    pub player_token_account: UncheckedAccount<'info>,

    /// Metadata account for the NFT (will be populated by the token metadata program)
    /// CHECK: Validated by the Metaplex token metadata program
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// Master edition account for the NFT
    /// CHECK: Validated by the Metaplex token metadata program
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    /// The Metaplex token metadata program
    /// CHECK:: metadata program
    pub metadata_program_info: AccountInfo<'info>,

    /// Instructions sysvar account required by the Metaplex token metadata program
    #[account(address = anchor_lang::solana_program::sysvar::instructions::id())]
    /// CHECK: Address is validated to be the instructions sysvar
    pub sysvar_instructions: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> MintPetNft<'info> {
    pub fn mint_pet_nft(&mut self, uri: String) -> Result<()> {
        let spl_token_program = &self.token_program.to_account_info();

        let cpi_accounts = CreateV1CpiAccounts {
            metadata: &self.metadata.to_account_info(),
            master_edition: Some(&self.master_edition),
            mint: (&self.nft_mint.to_account_info(), false),
            authority: &self.game_config.to_account_info(),
            payer: &self.player.to_account_info(),
            update_authority: (&self.game_config.to_account_info(), false),
            system_program: &self.system_program.to_account_info(),
            sysvar_instructions: &self.sysvar_instructions.to_account_info(),
            spl_token_program: Some(spl_token_program),
        };

        let cpi_args = CreateV1InstructionArgs {
            name: String::from("pet"),
            symbol: String::from("pet"),
            uri,
            seller_fee_basis_points: 0,
            creators: None,
            primary_sale_happened: false,
            is_mutable: true,
            token_standard: NonFungible,
            collection: Some(Collection {
                key: self.collection_mint.key(),
                verified: false,
            }),
            uses: None,
            collection_details: None,
            rule_set: None,
            decimals: Some(0),
            print_supply: Some(PrintSupply::Zero),
        };

        let signers_seeds: &[&[&[u8]]] = &[&[b"game_config", &[self.game_config.bump]]];

        CreateV1Cpi::new(&self.metadata_program_info, cpi_accounts, cpi_args)
            .invoke_signed(signers_seeds)?;

        let player = &self.player.to_account_info();
        let master_edition = &self.master_edition.to_account_info();

        let mint_accounts = MintV1CpiAccounts {
            token: &self.player_token_account.to_account_info(),
            token_owner: Some(player),
            metadata: &self.metadata.to_account_info(),
            master_edition: Some(master_edition),
            token_record: None,
            mint: &self.nft_mint.to_account_info(),
            authority: &self.game_config.to_account_info(),
            delegate_record: None,
            payer: &self.player.to_account_info(),
            system_program: &self.system_program.to_account_info(),
            spl_token_program: spl_token_program,
            spl_ata_program: &self.associated_token_program.to_account_info(),
            authorization_rules_program: None,
            authorization_rules: None,
            sysvar_instructions: &self.sysvar_instructions.to_account_info(),
        };

        let mint_cpi_args = MintV1InstructionArgs {
            amount: 1,
            authorization_data: None,
        };

        MintV1Cpi::new(&self.metadata_program_info, mint_accounts, mint_cpi_args)
            .invoke_signed(signers_seeds)?;
        Ok(())
    }
}
