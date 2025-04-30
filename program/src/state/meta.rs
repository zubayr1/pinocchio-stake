use super::{Authorized, Lockup};

#[repr(C)]
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Meta {
    rent_exempt_reserve: [u8; 8], // u64
    pub authorized: Authorized,
    pub lockup: Lockup,
}

impl Meta {
    #[inline(always)]
    pub fn set_rent_exempt_reserve(&mut self, rent_exempt_reserve: u64) {
        self.rent_exempt_reserve = rent_exempt_reserve.to_le_bytes();
    }

    #[inline(always)]
    pub fn rent_exempt_reserve(&self) -> u64 {
        u64::from_le_bytes(self.rent_exempt_reserve)
    }
}
