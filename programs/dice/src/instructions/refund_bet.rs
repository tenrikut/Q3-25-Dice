use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{error::DiceError, Bet, REFUND_TIMEOUT_SLOTS};

/// Refund Bet Instruction - Allows players to recover funds from unresolved bets
///
/// This instruction provides a safety mechanism for players when their bets
/// are not resolved by the house within a reasonable timeframe. After the
/// timeout period expires, players can reclaim their bet amount.
#[derive(Accounts)]
pub struct RefundBet<'info> {
    /// The player requesting the refund
    /// Must be the same player who originally placed the bet
    #[account(mut)]
    pub player: Signer<'info>,

    /// House authority (unchecked for efficiency)
    /// Used only for vault PDA seed derivation
    /// The bet account constraints ensure only the correct player can refund
    ///CHECK: This check is safe - house authority for vault seeds
    pub house: UncheckedAccount<'info>,

    /// House vault containing the funds to be refunded
    /// Must have sufficient balance to cover the refund amount
    #[account(
        mut,
        seeds = [b"vault", house.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    /// The bet account to be refunded
    /// - Must belong to the requesting player (enforced by constraint)
    /// - Seeds ensure only the original player can access their bet
    /// - After refund, the bet is marked as resolved to prevent double-spending
    #[account(
        mut,
        seeds = [b"bet", player.key().as_ref(), bet.seed.to_le_bytes().as_ref()],
        bump = bet.bump,
        constraint = bet.player == player.key() @ DiceError::NotPlayerBet
    )]
    pub bet: Account<'info, Bet>,

    /// System program required for SOL transfers
    pub system_program: Program<'info, System>,
}

impl<'info> RefundBet<'info> {
    /// Process a refund request for an unresolved bet
    ///
    /// # Arguments
    /// * `bumps` - PDA bumps needed for vault signing authority
    ///
    /// # Returns
    /// * `Result<()>` - Success or validation error
    ///
    /// # Refund Eligibility Requirements
    /// 1. Bet must not already be resolved
    /// 2. Sufficient time (REFUND_TIMEOUT_SLOTS) must have passed since bet placement
    /// 3. Vault must have sufficient funds for the refund
    /// 4. Only the original player can request refund (enforced by account constraints)
    ///
    /// # Safety Mechanisms
    /// - Bet is marked as resolved after refund to prevent double-spending
    /// - Timeout prevents immediate refunds that could disrupt normal game flow
    /// - Vault balance check ensures refund won't fail due to insufficient funds
    pub fn refund_bet(&mut self, bumps: &RefundBetBumps) -> Result<()> {
        let bet = &mut self.bet;
        let clock = Clock::get()?;

        // VALIDATION: Check if bet is already resolved
        // Resolved bets (win/loss/previous refund) cannot be refunded again
        if bet.is_resolved == true {
            return Err(DiceError::BetAlreadyResolved.into());
        }

        // VALIDATION: Check if enough time has passed for refund eligibility
        // This prevents immediate refunds and gives the house reasonable time to resolve bets
        // REFUND_TIMEOUT_SLOTS is typically 150 slots (~1 minute on Solana)
        let slots_passed = clock.slot.saturating_sub(bet.commit_slot);
        if slots_passed < REFUND_TIMEOUT_SLOTS {
            return Err(DiceError::RefundNotEligible.into());
        }

        // VALIDATION: Check if vault has sufficient funds for the refund
        // This prevents runtime errors during the transfer operation
        if bet.amount > self.vault.to_account_info().lamports() {
            return Err(DiceError::InsufficientFunds.into());
        }

        // SETUP: Prepare vault PDA signing authority
        // The vault PDA must sign the transfer since it owns the funds
        let house_key = self.house.key();
        let seeds = &[b"vault", house_key.as_ref(), &[bumps.vault]];
        let signer = &[&seeds[..]];

        // TRANSFER: Return the bet amount from vault back to player
        let accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.player.to_account_info(),
        };

        let ctx =
            CpiContext::new_with_signer(self.system_program.to_account_info(), accounts, signer);

        transfer(ctx, bet.amount)?;

        // FINALIZATION: Mark the bet as resolved to prevent double-spending
        // This ensures the bet cannot be refunded again or resolved normally
        bet.is_resolved = true;

        Ok(())
    }
}
