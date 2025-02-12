pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("2fiEN7yCZn3tQ3zVJ2iKTv7Tw5qkWU7gPsccZDvTZXQk");

#[program]
pub mod solapet_capstone {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }
}
