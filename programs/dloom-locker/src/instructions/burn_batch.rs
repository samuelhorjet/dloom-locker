// FILE: programs/dloom_locker/src/instructions/burn_batch.rs
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Burn, TokenInterface};
use crate::{errors::LockerError, events::BatchTokensBurned};

pub fn handle_burn_batch<'info>(
    ctx: Context<'_, '_, '_, 'info, BurnBatch<'info>>, 
    amounts: Vec<u64>
) -> Result<()> {
    let mut burned_mints = Vec::new();
    let remaining_accs = ctx.remaining_accounts;

    // Each burn operation requires a mint and a token account (2 accounts per amount).
    if remaining_accs.len() != amounts.len() * 2 {
        return err!(LockerError::InvalidBatchAccounts);
    }

    for (i, amount) in amounts.iter().enumerate() {
        require!(*amount > 0, LockerError::ZeroAmount);
        
        // Extract the mint and user token account from the remaining_accounts list.
        let mint_info = &remaining_accs[i * 2];
        let user_token_account_info = &remaining_accs[i * 2 + 1];

        // Perform the burn operation directly.
        token_interface::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: mint_info.clone(),
                    from: user_token_account_info.clone(),
                    authority: ctx.accounts.burner.to_account_info(),
                },
            ),
            *amount
        )?;

        // Record the mint's public key for the event log.
        burned_mints.push(mint_info.key());
    }

    // Emit an event to record the entire batch burn operation.
    emit!(BatchTokensBurned {
        burner: ctx.accounts.burner.key(),
        mints: burned_mints,
        amounts,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct BurnBatch<'info> {
    #[account(mut)]
    pub burner: Signer<'info>,
    pub token_program: Interface<'info, TokenInterface>,
}