use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};
use constant_product_curve::{ConstantProduct, LiquidityPair};

use crate::{errors::AmmError, state::Config};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub swapper: Signer<'info>,

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
        associated_token::mint = mint_x,
        associated_token::authority = swapper,
    )]
    pub user_x: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = swapper,
        associated_token::mint = mint_y,
        associated_token::authority = swapper,
    )]
    pub user_y: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Swap<'info> {
    pub fn swap(&mut self, is_x: bool, amount: u64, min: u64) -> Result<()> {
        require!(self.config.locked == false, AmmError::PoolLocked);
        require!(amount != 0, AmmError::InvalidAmount);

        let (to_vault_amount, from_vault_amount) = match is_x {
            true => (self.vault_x.amount, self.vault_y.amount),
            false => (self.vault_y.amount, self.vault_x.amount),
        };

        let fee_amount = (amount as u128)
            .checked_mul(self.config.fee as u128)
            .ok_or(AmmError::Overflow)?
            .checked_div(10_000)
            .ok_or(AmmError::Underflow)? as u64;
        let swap_in = (amount as u128)
            .checked_sub(fee_amount as u128)
            .ok_or(AmmError::Underflow)? as u64;

        let k = (to_vault_amount as u128)
            .checked_mul(from_vault_amount as u128)
            .ok_or(AmmError::Overflow)?;
        let new_x = (to_vault_amount as u128)
            .checked_add(swap_in as u128)
            .ok_or(AmmError::Overflow)? as u64;
        let new_y = k.checked_div(new_x as u128).ok_or(AmmError::Underflow)? as u64;
        let swap_out = from_vault_amount
            .checked_sub(new_y)
            .ok_or(AmmError::Underflow)?;

        require!(swap_out >= min, AmmError::SlippageExceeded);
        require!(swap_out <= from_vault_amount, AmmError::InsufficientBalance);

        self.deposit_tokens(is_x, amount)?;
        self.withdraw_tokens(is_x, swap_out)?;

        Ok(())
    }

    pub fn deposit_tokens(&mut self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (
                self.user_x.to_account_info(),
                self.vault_x.to_account_info(),
            ),
            false => (
                self.user_y.to_account_info(),
                self.vault_y.to_account_info(),
            ),
        };

        let transfer_accounts = Transfer {
            from,
            to,
            authority: self.swapper.to_account_info(),
        };

        transfer(
            CpiContext::new(self.token_program.to_account_info(), transfer_accounts),
            amount,
        )
    }

    pub fn withdraw_tokens(&mut self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (
                self.vault_y.to_account_info(),
                self.user_y.to_account_info(),
            ),
            false => (
                self.vault_x.to_account_info(),
                self.user_x.to_account_info(),
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
}
