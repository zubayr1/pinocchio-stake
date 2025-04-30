use pinocchio::pubkey::Pubkey;

use super::Epoch;

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Delegation {
    /// to whom the stake is delegated
    pub voter_pubkey: Pubkey,
    /// activated stake amount, set at delegate() time
    stake: [u8; 8], // u64
    /// epoch at which this stake was activated, std::Epoch::MAX if is a bootstrap stake
    activation_epoch: Epoch,
    /// epoch the stake was deactivated, std::Epoch::MAX if not deactivated
    deactivation_epoch: Epoch,
    /// how much stake we can activate per-epoch as a fraction of currently effective stake
    #[deprecated(
        since = "1.16.7",
        note = "Please use `solana_sdk::stake::state::warmup_cooldown_rate()` instead"
    )]
    warmup_cooldown_rate: [u8; 8], //f64
}

impl Delegation {
    #[inline(always)]
    pub fn set_stake(&mut self, stake: u64) {
        self.stake = stake.to_le_bytes();
    }

    #[inline(always)]
    pub fn stake(&self) -> u64 {
        u64::from_le_bytes(self.stake)
    }

    #[inline(always)]
    pub fn set_activation_epoch(&mut self, activation_epoch: u64) {
        self.activation_epoch = activation_epoch.to_le_bytes();
    }

    #[inline(always)]
    pub fn activation_epoch(&self) -> u64 {
        u64::from_le_bytes(self.activation_epoch)
    }

    #[inline(always)]
    pub fn set_deactivation_epoch(&mut self, deactivation_epoch: u64) {
        self.deactivation_epoch = deactivation_epoch.to_le_bytes();
    }

    #[inline(always)]
    pub fn deactivation_epoch(&self) -> u64 {
        u64::from_le_bytes(self.deactivation_epoch)
    }
}

pub const DEFAULT_WARMUP_COOLDOWN_RATE: f64 = 0.25;

impl Default for Delegation {
    fn default() -> Self {
        #[allow(deprecated)]
        Self {
            voter_pubkey: Pubkey::default(),
            stake: 0u64.to_le_bytes(),
            activation_epoch: 0u64.to_le_bytes(),
            deactivation_epoch: u64::MAX.to_le_bytes(),
            warmup_cooldown_rate: DEFAULT_WARMUP_COOLDOWN_RATE.to_le_bytes(),
        }
    }
}
