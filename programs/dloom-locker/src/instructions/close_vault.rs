// FILE: programs/dloom_locker/src/instructions/close_vault.rs
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, CloseAccount, TokenAccount, TokenInterface, Mint};
use anchor_spl::token_2022::spl_token_2022::extension::BaseStateWithExtensions; 
use anchor_spl::token_2022::spl_token_2022::{
    extension::{transfer_fee::TransferFeeAmount, StateWithExtensions},
    state::Account as Token2022Account,
};
use crate::{errors::LockerError, state::LockRecord};

pub fn handle_close_vault(ctx: Context<CloseVault>, lock_id: u64) -> Result<()> {
    require!(ctx.accounts.lock_record.amount == 0, LockerError::ZeroAmount);

    // Check for withheld fees in Token-2022
    let has_fees = {
        let vault_info = ctx.accounts.vault.to_account_info();
        let vault_data = vault_info.try_borrow_data()?;
        if let Ok(state) = StateWithExtensions::<Token2022Account>::unpack(&vault_data) {
            // This .get_extension() call requires BaseStateWithExtensions to be in scope
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
        return err!(LockerError::CannotCloseWithheldFees); 
    }

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

    token_interface::close_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        CloseAccount {
            account: ctx.accounts.vault.to_account_info(),
            destination: ctx.accounts.owner.to_account_info(),
            authority: ctx.accounts.lock_record.to_account_info(),
        },
        signer_seeds,
    ))?;

    msg!("Vault closed successfully.");

    Ok(())
}

#[derive(Accounts)]
#[instruction(lock_id: u64)]
pub struct CloseVault<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        close = owner,
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

    pub token_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
}