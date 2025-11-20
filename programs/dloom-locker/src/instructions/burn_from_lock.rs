// FILE: programs/dloom_locker/src/instructions/burn_from_lock.rs
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Burn, Mint, TokenAccount, TokenInterface};
use crate::{errors::LockerError, events::LockedTokensBurned, state::LockRecord};

pub fn handle_burn_from_lock(ctx: Context<BurnFromLock>, amount: u64, lock_id: u64) -> Result<()> {
    require!(amount > 0, LockerError::ZeroAmount);
    require!(ctx.accounts.lock_record.amount >= amount, LockerError::BurnAmountExceedsLocked);

    // Prepare seeds for signing
    let owner_key = ctx.accounts.owner.key();
    let mint_key = ctx.accounts.token_mint.key();
    let lock_id_bytes = lock_id.to_le_bytes();
    let bump = ctx.accounts.lock_record.bump;

    let seeds = &[
        b"lock_record".as_ref(),
        owner_key.as_ref(),
        mint_key.as_ref(),
        lock_id_bytes.as_ref(),
        &[bump],
    ];
    let signer_seeds = &[&seeds[..]];

    // Burn from Vault
    token_interface::burn(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.token_mint.to_account_info(),
                from: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.lock_record.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
    )?;

    // Update State
    ctx.accounts.lock_record.amount = ctx.accounts.lock_record.amount.checked_sub(amount).unwrap();

    emit!(LockedTokensBurned {
        owner: ctx.accounts.owner.key(),
        mint: ctx.accounts.token_mint.key(),
        amount,
        lock_id
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(amount: u64, lock_id: u64)]
pub struct BurnFromLock<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        has_one = owner,
        has_one = vault,
        seeds = [
            b"lock_record", 
            owner.key().as_ref(), 
            token_mint.key().as_ref(), 
            &lock_id.to_le_bytes()
        ],
        bump = lock_record.bump
    )]
    pub lock_record: Account<'info, LockRecord>,

    #[account(
        mut,
        token::mint = token_mint,
        token::authority = lock_record,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}