//! # Dice Game - Solana Anchor Program
//!
//! A decentralized dice betting game built on Solana using the Anchor framework.
//! Players can place bets on dice roll outcomes with provably fair randomness
//! using Ed25519 signatures for secure random number generation.

#![allow(unexpected_cfgs)]
#![allow(deprecated)]

// Module declarations for the dice game program
pub mod constants; // Game configuration and betting limits
pub mod error; // Custom error definitions for the program
pub mod instructions; // All instruction handlers (initialize, place_bet, resolve_bet, refund_bet)
pub mod state; // Data structures and account definitions

use anchor_lang::prelude::*;

// Re-export all modules for easier access
pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("CV4X2KEEEv9PEmPwH1Uk1kL6vw7mTpMBBduTp713ZcU3");

#[program]
pub mod dice_game {
    use super::*;

    /// Initialize the game vault with initial funds from the house
    ///
    /// # Arguments
    /// * `ctx` - Context containing accounts needed for initialization
    /// * `amount` - Initial amount in lamports to fund the vault
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn initialize(ctx: Context<Initialize>, amount: u64) -> Result<()> {
        ctx.accounts.init(amount)
    }

    /// Place a new bet on a dice roll outcome
    ///
    /// # Arguments
    /// * `ctx` - Context containing all required accounts
    /// * `seed` - Unique seed to allow multiple bets from same player
    /// * `roll` - Player's prediction (2-96, higher numbers = higher payout)
    /// * `amount` - Bet amount in lamports
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Game Logic
    /// Player wins if the random roll is LESS than their predicted number.
    /// Higher predictions = higher chance of winning but lower payout multiplier.
    pub fn place_bet(ctx: Context<PlaceBet>, seed: u128, roll: u8, amount: u64) -> Result<()> {
        ctx.accounts.create_bet(
            amount,
            roll,
            seed,
            ctx.accounts.randomness_account.key(),
            &ctx.bumps,
        )
    }

    /// Resolve a placed bet using Ed25519 signature for randomness
    ///
    /// # Arguments
    /// * `ctx` - Context containing bet and vault accounts
    /// * `sig` - Ed25519 signature bytes used to generate random number
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Security
    /// The signature is verified to ensure it comes from the house authority
    /// and is used as entropy source for provably fair randomness.
    pub fn resolve_bet(ctx: Context<ResolveBet>, sig: Vec<u8>) -> Result<()> {
        ctx.accounts.verify_ed25519_signature(&sig)?;
        ctx.accounts.resolve_bet(&ctx.bumps, &sig)
    }

    /// Refund a bet that hasn't been resolved within the timeout period
    ///
    /// # Arguments
    /// * `ctx` - Context containing bet and vault accounts
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Refund Policy
    /// Players can claim refunds if their bet hasn't been resolved
    /// after REFUND_TIMEOUT_SLOTS (~1 minute) have passed.
    pub fn refund_bet(ctx: Context<RefundBet>) -> Result<()> {
        ctx.accounts.refund_bet(&ctx.bumps)
    }
}
