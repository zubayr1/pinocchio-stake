use super::Delegation;

#[repr(C)]
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Stake {
    pub delegation: Delegation,
    /// credits observed is credits from vote account state when delegated or redeemed
    credits_observed: [u8; 8], //u64
}


impl Stake {
    #[inline(always)]
    pub fn set_credits_observed(&mut self, credits_observed: u64) {
        self.credits_observed = credits_observed.to_le_bytes();
    }

    #[inline(always)]
    pub fn credits_observed(&self) -> u64 {
        u64::from_le_bytes(self.credits_observed)
    }
}
