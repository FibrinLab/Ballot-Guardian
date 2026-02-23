use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod quadratic_voting {
    use super::*;

    pub fn placeholder(_ctx: Context<Placeholder>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Placeholder {}

