// FILE: programs/dloom_locker/src/instructions/mod.rs
pub mod burn_from_wallet;
pub mod burn_from_lock;
pub mod lock_tokens;
pub mod withdraw_tokens;
pub mod close_vault; 

pub use burn_from_wallet::*;
pub use burn_from_lock::*;
pub use lock_tokens::*;
pub use withdraw_tokens::*;
pub use close_vault::*;