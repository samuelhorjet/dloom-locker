// FILE: programs/dloom_locker/src/instructions/burn_tokens.rs
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Burn, Mint, TokenAccount, TokenInterface};
use crate::{errors::LockerError, events::TokensBurned};

pub fn handle_burn_tokens(ctx: Context<BurnTokens>, amount: u64) -> Result<()> {
    require!(amount > 0, LockerError::ZeroAmount);

    token_interface::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.token_mint.to_account_info(),
                from: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.burner.to_account_info(),
            },
        ),
        amount,
    )?;

    emit!(TokensBurned {
        burner: ctx.accounts.burner.key(),
        mint: ctx.accounts.token_mint.key(),
        amount,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct BurnTokens<'info> {
    #[account(mut)]
    pub burner: Signer<'info>,

    #[account(mut)]
    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut, 
        constraint = user_token_account.owner == burner.key(),
        constraint = user_token_account.mint == token_mint.key()
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,
    
    pub token_program: Interface<'info, TokenInterface>,
}