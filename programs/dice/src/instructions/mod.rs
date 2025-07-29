// Instruction Modules for Dice Game Program
// ========================================
//
// This module organizes all the instruction handlers for the dice betting game.
// Each instruction represents a different operation that can be performed:
//
// 1. initialize  - Set up the house vault with initial funding
// 2. place_bet   - Players place new bets with their predictions
// 3. resolve_bet - House resolves bets using Ed25519 signatures for randomness
// 4. refund_bet  - Players can claim refunds for unresolved bets after timeout
//
// The instructions follow Solana's Account-based programming model where
// each instruction specifies exactly which accounts it needs and how they
// should be validated (seeds, constraints, mutability, etc.).

pub mod initialize;
pub mod place_bet;
pub mod refund_bet;
pub mod resolve_bet;

// Re-export all instruction types for easy access from the main program
pub use initialize::*;
pub use place_bet::*;
pub use refund_bet::*;
pub use resolve_bet::*;
