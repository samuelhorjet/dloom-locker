// FILE: programs/dloom_locker/src/state/lock_record.rs
use anchor_lang::prelude::*;

#[account]
pub struct LockRecord {
    pub bump: u8,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub amount: u64,
    pub unlock_timestamp: i64,
    pub id: u64, 
}