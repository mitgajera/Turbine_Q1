use anchor_lang::prelude::*;

declare_id!("2PLpck15JpdjvbTFzSegbzAUGSowxgPXtm73fm2A2hYM");

#[program]
pub mod anchor_voting {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
