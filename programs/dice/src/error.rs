use anchor_lang::prelude::*;

#[error_code]
pub enum DiceError {
    #[msg("Bet has already been placed")]
    BetAlreadyPlaced,
    #[msg("Bet has already been resolved")]
    BetAlreadyResolved,
    #[msg("Failed to parse randomness data")]
    FailedToParseRandomness,
    #[msg("Randomness has expired")]
    RandomnessExpired,
    #[msg("Randomness not resolved")]
    RandomnessNotResolved,
    #[msg("Insufficient funds in vault")]
    InsufficientFunds,
    #[msg("Bet does not belong to the player")]
    NotPlayerBet,
    #[msg("Refund not yet eligible - wait more slots")]
    RefundNotEligible,
    #[msg("Bet amount below minimum")]
    MinimumBet,
    #[msg("Bet amount above maximum")]
    MaximumBet,
    #[msg("Roll prediction below minimum")]
    MinimumRoll,
    #[msg("Roll prediction above maximum")]
    MaximumRoll,
    #[msg("Invalid Ed25519 program")]
    Ed25519Program,
    #[msg("Ed25519 instruction should have no accounts")]
    Ed25519Accounts,
    #[msg("Invalid Ed25519 data length")]
    Ed25519DataLength,
    #[msg("Invalid Ed25519 header")]
    Ed25519Header,
    #[msg("Invalid Ed25519 public key")]
    Ed25519Pubkey,
    #[msg("Invalid Ed25519 signature")]
    Ed25519Signature,
    #[msg("Arithmetic overflow")]
    Overflow,
}
