use anchor_instruction_sysvar::Ed25519InstructionSignatures;

use anchor_lang::{
    prelude::*,
    solana_program::{ed25519_program, sysvar::instructions::load_instruction_at_checked},
};

use crate::error::ErrorCode;

pub fn verify_ed25519_signature(instruction_account: &AccountInfo,sig: &[u8]) -> Result<()> {
    let ix = load_instruction_at_checked(0, &instruction_account)?;

    require_keys_eq!(ix.program_id, ed25519_program::ID, ErrorCode::InvalidSig);

    require_eq!(ix.accounts.len(), 0, ErrorCode::InvalidSig);

    let signatures = Ed25519InstructionSignatures::unpack(&ix.data)?.0;

    require_eq!(signatures.len(), 1, ErrorCode::InvalidSig);
    let signature = &signatures[0];

    require!(signature.is_verifiable, ErrorCode::InvalidSig);

    require!(signature.signature.unwrap().eq(sig), ErrorCode::InvalidSig);

    // require!(signature.message.as_ref().unwrap().eq(account_to_check.to_slice()), ErrorCode::InvalidSig);

    Ok(())
}
