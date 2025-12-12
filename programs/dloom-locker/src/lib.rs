// FILE: programs/dloom_locker/src/lib.rs
use anchor_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("AVfmdPiqXfc15Pt8PPRXxTP5oMs4D1CdijARiz8mFMFD"); 

#[program]
pub mod dloom_locker {
    use super::*;

    pub fn handle_lock_tokens(ctx: Context<LockTokens>, amount: u64, unlock_timestamp: i64, lock_id: u64) -> Result<()> {
        instructions::lock_tokens::handle_lock_tokens(ctx, amount, unlock_timestamp, lock_id)
    }

    pub fn handle_withdraw_tokens(ctx: Context<WithdrawTokens>, lock_id: u64, amount: u64,) -> Result<()> {
        instructions::withdraw_tokens::handle_withdraw_tokens(ctx, lock_id, amount)
    }

    pub fn handle_close_vault(ctx: Context<CloseVault>, lock_id: u64) -> Result<()> {
        instructions::close_vault::handle_close_vault(ctx, lock_id)
    }

    // Burns tokens from the user's connected wallet
    pub fn handle_burn_from_wallet(ctx: Context<BurnFromWallet>, amount: u64) -> Result<()> {
        instructions::burn_from_wallet::handle_burn_from_wallet(ctx, amount)
    }

    // Burns tokens currently sitting in the Lock Vault
    pub fn handle_burn_from_lock(ctx: Context<BurnFromLock>, amount: u64, lock_id: u64) -> Result<()> {
        instructions::burn_from_lock::handle_burn_from_lock(ctx, amount, lock_id)
    }
}