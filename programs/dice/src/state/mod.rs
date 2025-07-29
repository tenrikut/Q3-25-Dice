use anchor_lang::prelude::*;

/// Bet Account - Stores all information about a single dice bet
///
/// Each bet is a Program Derived Account (PDA) with seeds:
/// ["bet", player_pubkey, seed_bytes]
///
/// This allows players to have multiple concurrent bets using different seeds.
#[account]
#[derive(InitSpace)]
pub struct Bet {
    /// Amount wagered in lamports
    pub amount: u64,

    /// Public key of the player who placed this bet
    pub player: Pubkey,

    /// Solana slot number when the bet was placed
    /// Used for timeout calculations and ordering
    pub slot: u64,

    /// Unique seed provided by player to enable multiple concurrent bets
    /// Prevents collision when same player wants multiple active bets
    pub seed: u128,

    /// Player's roll prediction (2-96)
    /// Player wins if the actual random roll is LESS than this number
    /// Higher numbers = higher win probability but lower payout multiplier
    pub roll: u8,

    /// PDA bump for this bet account
    /// Used for signing transactions on behalf of this account
    pub bump: u8,

    /// Public key of the randomness account used for this bet
    /// Links this bet to a specific source of randomness
    pub randomness_account: Pubkey,

    /// Slot number when the bet was committed/finalized
    /// Used to calculate refund eligibility timeouts
    pub commit_slot: u64,

    /// Flag indicating whether this bet has been resolved
    /// Prevents double-spending and determines refund eligibility
    /// - false: Bet is active and awaiting resolution
    /// - true: Bet has been resolved (win/loss) or refunded
    pub is_resolved: bool,
}
