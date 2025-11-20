// FILE: programs/dloom_locker/src/events.rs
use anchor_lang::prelude::*;

#[event]
pub struct TokensLocked {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub unlock_timestamp: i64,
}

#[event]
pub struct TokensWithdrawn {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
}

#[event]
pub struct TokensBurned {
    pub burner: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
}

#[event]
pub struct LockedTokensBurned {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub lock_id: u64,
}