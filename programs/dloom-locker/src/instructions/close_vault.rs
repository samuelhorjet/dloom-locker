// FILE: programs/dloom_locker/src/instructions/close_vault.rs
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, CloseAccount, TokenAccount, TokenInterface};
use anchor_spl::token_2022::spl_token_2022::{
    extension::{transfer_fee::TransferFeeAmount, StateWithExtensions, BaseStateWithExtensions},
    state::Account as Token2022Account,
};
use crate::{errors::LockerError, state::LockRecord};

pub fn handle_close_vault(ctx: Context<CloseVault>, lock_id: u64) -> Result<()> {
    require!(ctx.accounts.lock_record.amount == 0, LockerError::ZeroAmount);

    let has_fees = {
        let vault_info = ctx.accounts.vault.to_account_info();
        let vault_data = vault_info.try_borrow_data()?;
        if let Ok(state) = StateWithExtensions::<Token2022Account>::unpack(&vault_data) {
            if let Ok(extension) = state.get_extension::<TransferFeeAmount>() {
                u64::from(extension.withheld_amount) > 0
            } else {
                false
            }
        } else {
            false
        }
    };

    if has_fees {
        return err!(LockerError::StillLocked); 
    }

    // --- FIX: Copy values to local variables ---
    let owner_key = ctx.accounts.lock_record.owner;
    let mint_key = ctx.accounts.lock_record.mint;
    let bump = ctx.accounts.lock_record.bump;
    let lock_id_bytes = lock_id.to_le_bytes(); 

    let seeds = &[
        b"lock_record".as_ref(),
        owner_key.as_ref(),
        mint_key.as_ref(),
        lock_id_bytes.as_ref(),
        &[bump],
    ];
    let signer_seeds = &[&seeds[..]];
    // -------------------------------------------

    // 1. Close Vault
    token_interface::close_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        CloseAccount {
            account: ctx.accounts.vault.to_account_info(),
            destination: ctx.accounts.owner.to_account_info(),
            authority: ctx.accounts.lock_record.to_account_info(),
        },
        signer_seeds,
    ))?;

    // 2. Close LockRecord
    let dest = ctx.accounts.owner.to_account_info();
    let source = ctx.accounts.lock_record.to_account_info();
    **dest.try_borrow_mut_lamports()? = dest.lamports().checked_add(source.lamports()).unwrap();
    **source.try_borrow_mut_lamports()? = 0;

    msg!("Vault and LockRecord closed successfully.");

    Ok(())
}

#[derive(Accounts)]
#[instruction(lock_id: u64)]
pub struct CloseVault<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        has_one = owner,
        seeds = [
            b"lock_record", 
            owner.key().as_ref(), 
            token_mint.key().as_ref(),
            &lock_id.to_le_bytes()
        ],
        bump = lock_record.bump
    )]
    pub lock_record: Account<'info, LockRecord>,

    #[account(mut, address = lock_record.vault)]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: We only need the mint address to verify the seeds
    pub token_mint: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}