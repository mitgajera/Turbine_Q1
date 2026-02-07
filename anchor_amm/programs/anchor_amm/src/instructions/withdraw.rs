use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{burn, transfer, Burn, Mint, Token, TokenAccount, Transfer},
};
use constant_product_curve::ConstantProduct;

use crate::{errors::AmmError, state::Config};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub withdrawer: Signer<'info>,

    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump,
    )]
    pub mint_lp: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_lp,
        associated_token::authority = withdrawer,
    )]
    pub user_lp: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = withdrawer,
        associated_token::mint = mint_x,
        associated_token::authority = withdrawer,
    )]
    pub user_x: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = withdrawer,
        associated_token::mint = mint_y,
        associated_token::authority = withdrawer,
    )]
    pub user_y: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(
        &mut self,
        amount: u64,
        min_x: u64,
        min_y: u64,
    ) -> Result<()> {
        require!(amount != 0, AmmError::ZeroBalance);
        require!(amount <= self.user_lp.amount, AmmError::InsufficientBalance);

        let vault_x_amount = self.vault_x.amount;
        let vault_y_amount = self.vault_y.amount;
        let lp_supply = self.mint_lp.supply;

        let token_x = amount
            .checked_mul(vault_x_amount)
            .ok_or(AmmError::Overflow)?
            .checked_div(lp_supply)
            .ok_or(AmmError::Underflow)?;

        let token_y = amount
            .checked_mul(vault_y_amount)
            .ok_or(AmmError::Overflow)?
            .checked_div(lp_supply)
            .ok_or(AmmError::Underflow)?;

        require!(
            token_x >= min_x && token_y >= min_y,
            AmmError::SlippageExceeded
        );

        self.withdraw_tokens(true, token_x)?;
        self.withdraw_tokens(false, token_y)?;
        self.burn_lp_tokens(amount)?;

        Ok(())
    }

    pub fn withdraw_tokens(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (
                self.vault_x.to_account_info(),
                self.user_x.to_account_info(),
            ),
            false => (
                self.vault_y.to_account_info(),
                self.user_y.to_account_info(),
            ),
        };

        let transfer_accounts = Transfer {
            from,
            to,
            authority: self.config.to_account_info(),
        };

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"config",
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ]];

        transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                transfer_accounts,
                signer_seeds,
            ),
            amount,
        )
    }

    pub fn burn_lp_tokens(&self, amount: u64) -> Result<()> {
        let burn_accounts = Burn {
            mint: self.mint_lp.to_account_info(),
            from: self.user_lp.to_account_info(),
            authority: self.withdrawer.to_account_info(),
        };

        burn(
            CpiContext::new(self.token_program.to_account_info(), burn_accounts),
            amount,
        )
    }
}
