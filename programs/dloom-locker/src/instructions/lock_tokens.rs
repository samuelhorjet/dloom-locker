// FILE: programs/dloom_locker/src/instructions/lock_tokens.rs
use crate::{errors::LockerError, events::TokensLocked, state::LockRecord};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};

const MAX_LOCK_DURATION: i64 = 5 * 365 * 24 * 60 * 60;

pub fn handle_lock_tokens(
    ctx: Context<LockTokens>,
    amount: u64,
    unlock_timestamp: i64,
    lock_id: u64, 
) -> Result<()> {
    require!(amount > 0, LockerError::ZeroAmount);
    let current_timestamp = Clock::get()?.unix_timestamp;
    require!(
        unlock_timestamp > current_timestamp,
        LockerError::UnlockDateInPast
    );
    require!(
        unlock_timestamp - current_timestamp <= MAX_LOCK_DURATION,
        LockerError::LockDurationTooLong
    );

    // 1. Check balance BEFORE transfer
    ctx.accounts.vault.reload()?;
    let balance_before = ctx.accounts.vault.amount;

    // 2. Perform Transfer
    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
            },
        ),
        amount,
        ctx.accounts.token_mint.decimals,
    )?;

    ctx.accounts.vault.reload()?;
    let balance_after = ctx.accounts.vault.amount;
    
    // --- FIX: Removed unwrap() ---
    let actual_amount = balance_after
        .checked_sub(balance_before)
        .ok_or(LockerError::MathOverflow)?;

    let lock_record = &mut ctx.accounts.lock_record;
    lock_record.bump = ctx.bumps.lock_record;
    lock_record.owner = ctx.accounts.owner.key();
    lock_record.mint = ctx.accounts.token_mint.key();
    lock_record.vault = ctx.accounts.vault.key();
    lock_record.amount = actual_amount;
    lock_record.unlock_timestamp = unlock_timestamp;
    lock_record.id = lock_id; 

    emit!(TokensLocked {
        owner: lock_record.owner,
        mint: lock_record.mint,
        amount: actual_amount,
        unlock_timestamp: lock_record.unlock_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(amount: u64, unlock_timestamp: i64, lock_id: u64)]
pub struct LockTokens<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = owner,
        space = 8 + 1 + 32 + 32 + 32 + 8 + 8 + 8, 
        seeds = [
            b"lock_record", 
            owner.key().as_ref(), 
            token_mint.key().as_ref(), 
            &lock_id.to_le_bytes()
        ],
        bump
    )]
    pub lock_record: Account<'info, LockRecord>,

    #[account(
        init,
        payer = owner,
        seeds = [b"vault", lock_record.key().as_ref()],
        bump,
        token::mint = token_mint,
        token::authority = lock_record,
        token::token_program = token_program, 
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut, 
        constraint = user_token_account.mint == token_mint.key()
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub rent: Sysvar<'info, Rent>,
}