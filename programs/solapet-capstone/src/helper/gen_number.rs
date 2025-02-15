use anchor_lang::{prelude::*, solana_program::hash::hash};

pub fn gen_number(sig: &[u8], upper_cap: u128) -> Result<u8> {
    let hash = hash(sig).to_bytes();

    let mut hash_16: [u8; 16] = [0; 16];
    hash_16.copy_from_slice(&hash[0..16]);
    let lower = u128::from_le_bytes(hash_16);

    hash_16.copy_from_slice(&hash[16..32]);
    let upper = u128::from_le_bytes(hash_16);

    let roll = lower.wrapping_add(upper).wrapping_rem(upper_cap) as u8 + 1;

    Ok(roll)
}
