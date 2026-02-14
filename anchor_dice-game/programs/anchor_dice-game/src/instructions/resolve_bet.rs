use anchor_lang::{
    prelude::*,
    solana_program::{self, sysvar::instructions as sysvar_instructions},
    system_program::{transfer, Transfer},
};

use crate::{errors::DiceError, state::Bet};

#[derive(Accounts)]
#[instruction(seed: u128)]
pub struct ResolveBet<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    /// CHECK: House is only used as a seed and to receive potential profits
    pub house: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"vault", house.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        mut,
        close = player,
        seeds = [b"bet", vault.key().as_ref(), bet.seed.to_le_bytes().as_ref()],
        bump = bet.bump,
        has_one = player,
    )]
    pub bet: Account<'info, Bet>,

    /// CHECK: Instructions sysvar; address is constrained
    #[account(address = sysvar_instructions::ID)]
    pub instructions: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> ResolveBet<'info> {

    pub fn verify_ed25519_signature(&self, sig: &[u8]) -> Result<()> {
        let ix_sysvar_info = self.instructions.to_account_info();

        let ix = sysvar_instructions::load_instruction_at_checked(0, &ix_sysvar_info)
            .map_err(|_| DiceError::Ed25519Accounts)?;

        let data = ix.data.as_slice();

        require!(data.len() >= sig.len(), DiceError::Ed25519DataLength);
        require!(
            data.windows(sig.len()).any(|window| window == sig),
            DiceError::Ed25519Signature
        );

        Ok(())
    }

    /// Resolve the bet using the signature as entropy.
    pub fn resolve_bet(&mut self, sig: &[u8], bumps: &ResolveBetBumps) -> Result<()> {
        // Derive a pseudo-random roll in [1, 100] from bet data + signature.
        let mut msg = self.bet.to_slice();
        msg.extend_from_slice(sig);

        // Simple deterministic "hash": sum bytes and map to 1..=100.
        let sum: u8 = msg.iter().fold(0u8, |acc, b| acc.wrapping_add(*b));
        let outcome = (sum % 100) + 1; // 1..=100

        // Simple rule: player wins if outcome < bet.roll.
        let player_wins = outcome < self.bet.roll;

        if player_wins {
            // Pay out from the vault to the player; here we simply return the bet amount.
            let accounts = Transfer {
                from: self.vault.to_account_info(),
                to: self.player.to_account_info(),
            };

            let signer_seeds: &[&[&[u8]]] =
                &[&[b"vault", &self.house.key().to_bytes(), &[bumps.vault]]];

            let ctx = CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                accounts,
                signer_seeds,
            );

            transfer(ctx, self.bet.amount)
        } else {
            // House wins; funds remain in the vault.
            Ok(())
        }
    }
}

