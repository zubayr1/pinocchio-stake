use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
};

use crate::{consts::MAX_SIGNERS, error::StakeError};

pub trait DataLen {
    const LEN: usize;
}

pub trait Initialized {
    fn is_initialized(&self) -> bool;
}

#[inline(always)]
pub unsafe fn load_acc<T: DataLen + Initialized>(bytes: &[u8]) -> Result<&T, ProgramError> {
    load_acc_unchecked::<T>(bytes).and_then(|acc| {
        if acc.is_initialized() {
            Ok(acc)
        } else {
            Err(ProgramError::UninitializedAccount)
        }
    })
}

#[inline(always)]
pub unsafe fn load_acc_unchecked<T: DataLen>(bytes: &[u8]) -> Result<&T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(&*(bytes.as_ptr() as *const T))
}

#[inline(always)]
pub unsafe fn load_acc_mut<T: DataLen + Initialized>(
    bytes: &mut [u8],
) -> Result<&mut T, ProgramError> {
    load_acc_mut_unchecked::<T>(bytes).and_then(|acc| {
        if acc.is_initialized() {
            Ok(acc)
        } else {
            Err(ProgramError::UninitializedAccount)
        }
    })
}

#[inline(always)]
pub unsafe fn load_acc_mut_unchecked<T: DataLen>(bytes: &mut [u8]) -> Result<&mut T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(&mut *(bytes.as_mut_ptr() as *mut T))
}

#[inline(always)]
pub unsafe fn load_ix_data<T: DataLen>(bytes: &[u8]) -> Result<&T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(StakeError::InvalidInstructionData.into());
    }
    Ok(&*(bytes.as_ptr() as *const T))
}

pub unsafe fn to_bytes<T: DataLen>(data: &T) -> &[u8] {
    core::slice::from_raw_parts(data as *const T as *const u8, T::LEN)
}

pub unsafe fn to_mut_bytes<T: DataLen>(data: &mut T) -> &mut [u8] {
    core::slice::from_raw_parts_mut(data as *mut T as *mut u8, T::LEN)
}

//Stake Program Utils

pub fn collect_signers(
    accounts: &[AccountInfo],
    signers_arr: &mut [Pubkey; MAX_SIGNERS],
) -> Result<usize, ProgramError> {
    let mut signer_len = 0;

    for account in accounts {
        if account.is_signer() {
            if signer_len >= MAX_SIGNERS {
                return Err(ProgramError::AccountDataTooSmall);
            }
            signers_arr[signer_len] = *account.key();
            signer_len += 1;
        }
    }

    Ok(signer_len)
}

pub fn next_account_info<'a, I: Iterator<Item = &'a AccountInfo>>(
    iter: &mut I,
) -> Result<&'a AccountInfo, ProgramError> {
    iter.next().ok_or(ProgramError::NotEnoughAccountKeys)
}

#[macro_export]
macro_rules! impl_sysvar_id {
    ($type:ty) => {
        impl $crate::helpers::sysvar_stake_history::SysvarId for $type {
            fn id() -> $crate::Pubkey {
                id()
            }

            fn check_id(pubkey: &$crate::Pubkey) -> bool {
                check_id(pubkey)
            }
        }
    };
}

#[macro_export]
macro_rules! declare_sysvar_id(
    ($name:expr, $type:ty) => (
        pinocchio_pubkey::declare_id!($name);
        $crate::impl_sysvar_id!($type);
    )
);

/// After calling `validate_split_amount()`, this struct contains calculated
/// values that are used by the caller.
#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct ValidatedSplitInfo {
    pub source_remaining_balance: u64,
    pub destination_rent_exempt_reserve: u64,
}

/// Ensure the split amount is valid.  This checks the source and destination
/// accounts meet the minimum balance requirements, which is the rent exempt
/// reserve plus the minimum stake delegation, and that the source account has
/// enough lamports for the request split amount.  If not, return an error.
pub(crate) fn validate_split_amount(
    source_lamports: u64,
    destination_lamports: u64,
    split_lamports: u64,
    source_meta: &Meta,
    destination_data_len: usize,
    additional_required_lamports: u64,
    source_is_active: bool,
) -> Result<ValidatedSplitInfo, ProgramError> {
    // Split amount has to be something
    if split_lamports == 0 {
        return Err(ProgramError::InsufficientFunds);
    }

    // Obviously cannot split more than what the source account has
    if split_lamports > source_lamports {
        return Err(ProgramError::InsufficientFunds);
    }

    // Verify that the source account still has enough lamports left after
    // splitting: EITHER at least the minimum balance, OR zero (in this case the
    // source account is transferring all lamports to new destination account,
    // and the source account will be closed)
    let source_minimum_balance = source_meta
        .rent_exempt_reserve
        .saturating_add(additional_required_lamports);
    let source_remaining_balance = source_lamports.saturating_sub(split_lamports);
    if source_remaining_balance == 0 {
        // full amount is a withdrawal
        // nothing to do here
    } else if source_remaining_balance < source_minimum_balance {
        // the remaining balance is too low to do the split
        return Err(ProgramError::InsufficientFunds);
    } else {
        // all clear!
        // nothing to do here
    }

    let rent = Rent::get()?;
    let destination_rent_exempt_reserve = rent.minimum_balance(destination_data_len);

    // If the source is active stake, one of these criteria must be met:
    // 1. the destination account must be prefunded with at least the rent-exempt
    //    reserve, or
    // 2. the split must consume 100% of the source
    if source_is_active
        && source_remaining_balance != 0
        && destination_lamports < destination_rent_exempt_reserve
    {
        return Err(ProgramError::InsufficientFunds);
    }

    // Verify the destination account meets the minimum balance requirements
    // This must handle:
    // 1. The destination account having a different rent exempt reserve due to data
    //    size changes
    // 2. The destination account being prefunded, which would lower the minimum
    //    split amount
    let destination_minimum_balance =
        destination_rent_exempt_reserve.saturating_add(additional_required_lamports);
    let destination_balance_deficit =
        destination_minimum_balance.saturating_sub(destination_lamports);
    if split_lamports < destination_balance_deficit {
        return Err(ProgramError::InsufficientFunds);
    }

    Ok(ValidatedSplitInfo {
        source_remaining_balance,
        destination_rent_exempt_reserve,
    })
}
