pub mod constants;
pub mod error;
pub mod helper;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use helper::*;
pub use instructions::*;
pub use state::*;

declare_id!("2fiEN7yCZn3tQ3zVJ2iKTv7Tw5qkWU7gPsccZDvTZXQk");

#[program]
pub mod solapet_capstone {
    use super::*;

    pub fn initialize(ctx: Context<InitializeGameConfig>, fees: u8) -> Result<()> {
        ctx.accounts.init_game_config(&ctx.bumps, fees)?;
        Ok(())
    }

    pub fn update_fees(ctx: Context<UpdateFees>, fees: u8) -> Result<()> {
        ctx.accounts.update_fees(fees)?;
        Ok(())
    }

    pub fn init_player(ctx: Context<DepositNft>) -> Result<()> {
        ctx.accounts.deposit_nft()?;
        ctx.accounts.init_pet_stats(&ctx.bumps)?;
        Ok(())
    }

    pub fn close_player(ctx: Context<WithdrawNFT>) -> Result<()> {
        ctx.accounts.withdraw()?;
        Ok(())
    }

    pub fn pet_interact(
        ctx: Context<PetInteract>,
        interaction_type: InteractionType,
    ) -> Result<()> {
        ctx.accounts.interact(interaction_type)?;
        Ok(())
    }

    pub fn init_pet_duel(ctx: Context<InitPetDuel>, bet_amount: u64) -> Result<()> {
        ctx.accounts.initilize(&ctx.bumps, bet_amount)?;
        if bet_amount > 0 {
            ctx.accounts.deposite(bet_amount)?;
        }
        Ok(())
    }

    pub fn accept_pet_duel(ctx: Context<AcceptPetDuel>) -> Result<()> {
        ctx.accounts.accept_duel()?;
        if ctx.accounts.pet_duel_account.bet_amount > 0 {
            ctx.accounts.deposite()?;
        }
        Ok(())
    }

    pub fn pet_attack(ctx: Context<PetAttack>, sig: Vec<u8>) -> Result<()> {
        // verify_ed25519_signature(&ctx.accounts.instructions_sysvar.to_account_info(), &sig)?;
        ctx.accounts.attack(&sig)?;
        Ok(())
    }

    pub fn claim_bet(ctx: Context<ClaimBetAmount>) -> Result<()> {
        require!(
            ctx.accounts.pet_duel_account.winner.is_some()
                && ctx.accounts.winner.key() == ctx.accounts.pet_duel_account.winner.unwrap(),
            error::ErrorCode::UnauthorizedAction
        );
        if ctx.accounts.pet_duel_account.bet_amount > 0 {
            msg!("bet amount greater than zero, claming");
            ctx.accounts.claim()?;
        }
        Ok(())
    }
}
