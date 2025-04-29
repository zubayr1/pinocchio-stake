use pinocchio::pubkey::Pubkey;
use pinocchio_pubkey::pubkey;

pub const MAX_SIGNERS: usize = 32;
pub const FEATURE_STAKE_RAISE_MINIMUM_DELEGATION_TO_1_SOL: bool = false;
pub const PERPETUAL_NEW_WARMUP_COOLDOWN_RATE_EPOCH: Option<u64> = Some(0);
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
pub const SYSVAR: Pubkey = pubkey!("Sysvar1111111111111111111111111111111111111");
