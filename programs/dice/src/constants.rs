use anchor_lang::prelude::*;

// Base seed string used for generating program-derived addresses (PDAs)
#[constant]
pub const SEED: &str = "anchor";

// BETTING CONSTRAINTS
// ===================

/// Minimum bet amount: 0.01 SOL (10,000,000 lamports)
/// Prevents spam bets while keeping the game accessible
pub const MIN_BET_LAMPORTS: u64 = 10_000_000;

/// Maximum bet amount: 10 SOL (10,000,000,000 lamports)  
/// Limits maximum exposure and protects the house vault
pub const MAX_BET_LAMPORTS: u64 = 10_000_000_000;

/// Minimum roll prediction: 2
/// Players win if random roll (1-100) is LESS than their prediction
/// Minimum of 2 ensures there's always a chance to lose (if roll = 1)
pub const MIN_ROLL: u8 = 2;

/// Maximum roll prediction: 96
/// Maximum of 96 ensures there's always a chance to win (if roll = 97-100)
/// This creates a balanced risk/reward system
pub const MAX_ROLL: u8 = 96;

// GAME ECONOMICS
// ==============

/// House edge in basis points (150 = 1.5%)
/// This is the house's profit margin built into payouts
/// Example: On a winning bet, payout = (bet_amount * 98.5%) / (win_probability)
pub const HOUSE_EDGE: u16 = 150;

// TIMEOUT SETTINGS
// ================

/// Refund timeout: 150 slots (approximately 1 minute on Solana)
/// After this time passes without resolution, players can claim refunds
/// Protects players from stuck bets due to house inactivity
pub const REFUND_TIMEOUT_SLOTS: u64 = 150;
