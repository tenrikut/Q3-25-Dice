use anchor_instruction_sysvar::Ed25519InstructionSignatures;
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use solana_program::{
    ed25519_program, hash::hash, sysvar::instructions::load_instruction_at_checked,
};

use crate::{error::DiceError, state::Bet, HOUSE_EDGE};

/// Resolve Bet Instruction - Resolves a placed bet using Ed25519 signature for randomness
///
/// This instruction implements provably fair gambling by using Ed25519 signatures
/// as a source of randomness. The house provides a signature that is cryptographically
/// verified, then used to generate a fair random number for the dice roll.
#[derive(Accounts)]
pub struct ResolveBet<'info> {
    /// House authority that provides the Ed25519 signature for randomness
    /// Must sign this transaction to authorize the bet resolution
    #[account(mut)]
    pub house: Signer<'info>,

    /// Player who placed the bet (unchecked for efficiency)
    /// Will receive payout if they win the bet
    /// The bet account itself enforces that this matches the original player
    #[account(mut)]
    ///CHECK: This is safe
    pub player: UncheckedAccount<'info>,

    /// House vault containing funds for payouts
    /// Must match the PDA derived from house authority
    #[account(
        mut,
        seeds = [b"vault", house.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    /// The bet account to be resolved
    /// - Will be closed and rent returned to player after resolution
    /// - Must belong to the specified player (enforced by PDA seeds)
    /// - Bump must match the original bet creation
    #[account(
        mut,
        close = player,  // Close account and return rent to player
        seeds = [b"bet", player.key().as_ref(), bet.seed.to_le_bytes().as_ref()],
        bump = bet.bump
    )]
    pub bet: Account<'info, Bet>,

    /// Instruction sysvar account containing Ed25519 signature data
    /// Required for accessing the Ed25519 instruction that precedes this one
    #[account(
        address = solana_program::sysvar::instructions::ID
    )]
    /// CHECK: This is safe
    pub instruction_sysvar: AccountInfo<'info>,

    /// System program required for transferring payouts
    pub system_program: Program<'info, System>,
}

impl<'info> ResolveBet<'info> {
    /// Verify that the Ed25519 signature instruction is valid and properly formatted
    ///
    /// # Arguments
    /// * `sig` - The signature bytes that should match the Ed25519 instruction
    ///
    /// # Returns
    /// * `Result<()>` - Success if signature is valid, error otherwise
    ///
    /// # Security Requirements
    /// 1. The preceding instruction must be an Ed25519 verification instruction
    /// 2. The signature must be from the house authority
    /// 3. The message being signed must be the serialized bet data
    /// 4. No accounts should be present in the Ed25519 instruction
    pub fn verify_ed25519_signature(&mut self, sig: &[u8]) -> Result<()> {
        // Load the Ed25519 instruction that should precede this one
        // Index 0 refers to the instruction immediately before this one
        let ix = load_instruction_at_checked(0, &self.instruction_sysvar.to_account_info())?;

        // SECURITY: Ensure the instruction is addressed to the Ed25519 program
        require_keys_eq!(
            ix.program_id,
            ed25519_program::ID,
            DiceError::Ed25519Program
        );

        // SECURITY: Ed25519 verify instructions should not have any accounts
        require_eq!(ix.accounts.len(), 0, DiceError::Ed25519Accounts);

        // Parse the Ed25519 instruction data to extract signature information
        let signatures = Ed25519InstructionSignatures::unpack(&ix.data)?.0;

        // SECURITY: Should contain exactly one signature
        require_eq!(signatures.len(), 1, DiceError::Ed25519DataLength);
        let signature = &signatures[0];

        // SECURITY: All signature components must be verifiable (not None)
        require!(signature.is_verifiable, DiceError::Ed25519Header);

        // SECURITY: Public key must match the house authority
        require_keys_eq!(
            signature.public_key.ok_or(DiceError::Ed25519Pubkey)?,
            self.house.key(),
            DiceError::Ed25519Pubkey
        );

        // SECURITY: Signature bytes must match the provided signature
        require!(
            &signature
                .signature
                .ok_or(DiceError::Ed25519Signature)?
                .eq(sig),
            DiceError::Ed25519Signature
        );

        // SECURITY: Message must be the serialized bet data (prevents signature reuse)
        require!(
            &signature
                .message
                .as_ref()
                .ok_or(DiceError::Ed25519Signature)?
                .eq(&self.bet.try_to_vec()?),
            DiceError::Ed25519Signature
        );

        Ok(())
    }

    /// Resolve the bet by generating a random number and paying out winners
    ///
    /// # Arguments
    /// * `bumps` - PDA bumps needed for vault signing
    /// * `sig` - Ed25519 signature bytes used as entropy source
    ///
    /// # Returns
    /// * `Result<()>` - Success or payout error
    ///
    /// # Randomness Generation
    /// 1. Hash the Ed25519 signature to get 32 bytes of entropy
    /// 2. Split into two 16-byte chunks and convert to u128 integers
    /// 3. Add them together and take modulo 100 to get roll (1-100)
    ///
    /// # Payout Calculation
    /// If player wins: payout = (bet_amount * (100 - house_edge)) / (roll_prediction - 1) / 100
    /// The house edge is subtracted before calculating the odds-based payout.
    pub fn resolve_bet(&mut self, bumps: &ResolveBetBumps, sig: &[u8]) -> Result<()> {
        // RANDOMNESS: Generate provably fair random number from signature
        let hash = hash(sig).to_bytes();

        // Split the 32-byte hash into two 16-byte chunks
        let mut hash_16: [u8; 16] = [0; 16];
        hash_16.copy_from_slice(&hash[0..16]);
        let lower = u128::from_le_bytes(hash_16);
        hash_16.copy_from_slice(&hash[16..32]);
        let upper = u128::from_le_bytes(hash_16);

        // Combine the two halves and generate a roll from 1-100
        let roll = lower.wrapping_add(upper).wrapping_rem(100) as u8 + 1;

        // GAME LOGIC: Player wins if their prediction is HIGHER than the random roll
        if self.bet.roll > roll {
            // PAYOUT CALCULATION: Calculate winnings with house edge
            // Formula: (bet_amount * (10000 - house_edge_bp)) / (roll_prediction - 1) / 100
            // Example: 1 SOL bet on roll 50 = (1 * 9850) / 49 / 100 = ~2.01 SOL payout
            let payout = (self.bet.amount as u128)
                .checked_mul(10000 - HOUSE_EDGE as u128)
                .ok_or(DiceError::Overflow)? // Apply house edge
                .checked_div(self.bet.roll as u128 - 1)
                .ok_or(DiceError::Overflow)? // Odds-based multiplier
                .checked_div(100)
                .ok_or(DiceError::Overflow)? as u64; // Convert basis points

            // TRANSFER: Pay the winner from the house vault
            let accounts = Transfer {
                from: self.vault.to_account_info(),
                to: self.player.to_account_info(),
            };

            // Create PDA signer seeds for the vault
            let house_key = self.house.key();
            let seeds = [b"vault", house_key.as_ref(), &[bumps.vault]];
            let signer_seeds = &[&seeds[..]][..];

            // Execute the payout transfer using vault PDA as signer
            let ctx = CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                accounts,
                signer_seeds,
            );
            transfer(ctx, payout)?;
        }
        // If player loses (roll >= bet.roll), no payout is made
        // The bet amount stays in the vault as house profit

        Ok(())
    }
}
