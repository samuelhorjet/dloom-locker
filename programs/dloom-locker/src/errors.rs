// FILE: programs/dloom_locker/src/errors.rs
use anchor_lang::prelude::*;

#[error_code]
pub enum LockerError {
    #[msg("The unlock date must be in the future.")]
    UnlockDateInPast,
    #[msg("The amount to lock or burn must be greater than zero.")]
    ZeroAmount,
    #[msg("These tokens are still locked.")]
    StillLocked,
    #[msg("The lock duration is too long.")]
    LockDurationTooLong,
    #[msg("Vault has withheld transfer fees. Cannot close account until Token Admin harvests them.")]
    CannotCloseWithheldFees,
    #[msg("Burn amount exceeds locked amount.")]
    BurnAmountExceedsLocked,
    #[msg("Math operation overflow.")]
    MathOverflow, 
}