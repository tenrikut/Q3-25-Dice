use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

/// Initialize Instruction - Sets up the house vault for the dice game
///
/// This instruction must be called once by the house to fund the initial vault
/// that will be used to pay out winning bets. The vault is a Program Derived Account (PDA)
/// that can only be controlled by the program itself.
#[derive(Accounts)]
pub struct Initialize<'info> {
    /// The house authority that will fund the vault
    /// Must sign the transaction and have sufficient SOL for the initial deposit
    #[account(mut)]
    pub house: Signer<'info>,

    /// The house vault - a PDA that holds funds for paying out winning bets
    ///
    /// Seeds: ["vault", house_pubkey]
    /// - Ensures each house authority has a unique vault
    /// - The bump is automatically derived by Anchor
    /// - SystemAccount type allows direct lamport transfers
    #[account(
        mut,
        seeds = [b"vault",house.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    /// System program required for SOL transfers between accounts
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    /// Initialize the vault by transferring initial funds from house to vault
    ///
    /// # Arguments
    /// * `amount` - Amount in lamports to initially fund the vault
    ///
    /// # Returns
    /// * `Result<()>` - Success or anchor/system program error
    ///
    /// # Security Notes
    /// - Only the house can call this function (enforced by signer requirement)
    /// - The vault PDA ensures funds can only be withdrawn through program logic
    /// - Initial funding ensures the vault can pay out early winning bets
    pub fn init(&mut self, amount: u64) -> Result<()> {
        // Prepare the Cross-Program Invocation (CPI) accounts for the transfer
        let cpi_accounts = Transfer {
            from: self.house.to_account_info(),
            to: self.vault.to_account_info(),
        };

        // Create the CPI context for calling the system program
        let ctx = CpiContext::new(self.system_program.to_account_info(), cpi_accounts);

        // Execute the transfer from house to vault
        // This funds the vault so it can pay out winning bets
        transfer(ctx, amount)?;

        Ok(())
    }
}
