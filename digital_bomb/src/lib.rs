
use solana_program::declare_id;


pub mod entrypoint;
pub mod utils;
pub mod state;
pub mod instruction;
pub mod processor;


#[cfg(not(feature = "devnet"))]
declare_id!("GhXZtEtCGcz5sHyqPwCcJLXGKaJJgM3iA4ruRjBjaPQd");

#[cfg(feature = "devnet")]
declare_id!("GhXZtEtCGcz5sHyqPwCcJLXGKaJJgM3iA4ruRjBjaPQd");

pub mod constants {
        
    use solana_program::{pubkey, pubkey::Pubkey};
    pub const SYSTEM_ID: Pubkey = pubkey!("11111111111111111111111111111111");

    pub const VAULT: Pubkey = pubkey!("EYVjoX4t59WsHDoRMHySqHiG58zGbZCCieF5K9heLSc5");

    #[cfg(feature = "devnet")]
    pub const REVEAL_TIME: i64 = 6000; // 10min
    #[cfg(not(feature = "devnet"))]
    pub const REVEAL_TIME: i64 = 600000; // 1000min
}
