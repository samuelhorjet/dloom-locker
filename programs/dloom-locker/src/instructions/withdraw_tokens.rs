use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, CloseAccount, Mint, TokenAccount, TokenInterface, TransferChecked};
use anchor_spl::token_2022::spl_token_2022::extension::BaseStateWithExtensions;
use anchor_spl::token_2022::spl_token_2022::{
    extension::{transfer_fee::TransferFeeAmount, StateWithExtensions},
    state::Account as Token2022Account,
};
use crate::{errors::LockerError, events::TokensWithdrawn, state::LockRecord};

pub fn handle_withdraw_tokens(ctx: Context<WithdrawTokens>, lock_id: u64, amount: u64) -> Result<()> {
    require!(Clock::get()?.unix_timestamp >= ctx.accounts.lock_record.unlock_timestamp, LockerError::StillLocked);
    
    require!(amount > 0, LockerError::ZeroAmount);
    require!(amount <= ctx.accounts.lock_record.amount, LockerError::InsufficientFunds); // Ensure you define InsufficientFunds in errors.rs

    let owner_key = ctx.accounts.owner.key();
    let mint_key = ctx.accounts.token_mint.key();
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

    // 1. Transfer tokens back to user
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.lock_record.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
            },
            signer_seeds,
        ),
        amount, 
        ctx.accounts.token_mint.decimals,
    )?;

    // CHANGE 4: Update state - subtract withdrawn amount
    ctx.accounts.lock_record.amount = ctx.accounts.lock_record.amount.checked_sub(amount).unwrap();
    let remaining_amount = ctx.accounts.lock_record.amount;

    // 2. Check for Transfer Fees
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

    // 3. Conditional Close Logic
    // CHANGE 5: Only close if NO fees exist AND remaining balance is 0
    if !has_fees && remaining_amount == 0 {
        // A. Close Vault (SPL Account)
        token_interface::close_account(CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            CloseAccount {
                account: ctx.accounts.vault.to_account_info(),
                destination: ctx.accounts.owner.to_account_info(),
                authority: ctx.accounts.lock_record.to_account_info(),
            },
            signer_seeds,
        ))?;

        // B. Close LockRecord (Anchor Account)
        let dest = ctx.accounts.owner.to_account_info();
        let source = ctx.accounts.lock_record.to_account_info();
        
        let dest_starting_lamports = dest.lamports();
        **dest.try_borrow_mut_lamports()? = dest_starting_lamports.checked_add(source.lamports()).unwrap();
        **source.try_borrow_mut_lamports()? = 0;
        
        msg!("Lock fully withdrawn and accounts closed.");
    } else {
        if remaining_amount > 0 {
            msg!("Partial withdrawal complete. Remaining locked: {}", remaining_amount);
        } else {
            msg!("Vault has withheld fees. Accounts left open. User got principal tokens back.");
        }
    }

    emit!(TokensWithdrawn {
        owner: ctx.accounts.owner.key(),
        mint: ctx.accounts.lock_record.mint,
        amount: amount, 
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(lock_id: u64, amount: u64)] 
pub struct WithdrawTokens<'info> {
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

    #[account(mut)]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,
    
    pub token_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
}