use crate::helpers::*;
use pinocchio::{account_info::AccountInfo, pubkey::Pubkey, sysvars::clock::Clock, ProgramResult};

/*
use solana_program::{
    account_info::{next_account_info},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::set_return_data,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    stake::{
        instruction::{
            AuthorizeCheckedWithSeedArgs, AuthorizeWithSeedArgs, LockupArgs, LockupCheckedArgs,
            StakeError, StakeInstruction,
        },
        stake_flags::StakeFlags,
        state::{Authorized, Lockup, Meta, StakeAuthorize, StakeStateV2},
        tools::{acceptable_reference_epoch_credits, eligible_for_deactivate_delinquent},
    },
    sysvar::{epoch_rewards::EpochRewards, stake_history::StakeHistorySysvar, Sysvar},
    vote::{program as solana_vote_program, state::VoteState},
};
*/
use std::{collections::HashSet, mem::MaybeUninit};

// almost all native stake program processors accumulate every account signer
// they then defer all signer validation to functions on Meta or Authorized
// this results in an instruction interface that is much looser than the one documented
// to avoid breaking backwards compatibility, we do the same here
// in the future, we may decide to tighten the interface and break badly formed transactions
fn collect_signers(accounts: &[AccountInfo]) -> HashSet<Pubkey> {
    let mut signers = HashSet::new();

    for account in accounts {
        if account.is_signer() {
            signers.insert(*account.key());
        }
    }

    signers
}

pub fn process_split(accounts: &[AccountInfo], split_lamports: u64) -> ProgramResult {
    let signers = collect_signers(accounts);
    let account_info_iter = &mut accounts.iter();

    // native asserts: 2 accounts
    let source_stake_account_info = next_account_info(account_info_iter)?;
    let destination_stake_account_info = next_account_info(account_info_iter)?;

    // other accounts
    // let _stake_authority_info = next_account_info(account_info_iter);

    let clock = Clock::get()?;
    let stake_history = &StakeHistorySysvar(clock.epoch);

    let destination_data_len = destination_stake_account_info.data_len();
    if destination_data_len != StakeStateV2::size_of() {
        return Err(ProgramError::InvalidAccountData);
    }

    if let StakeStateV2::Uninitialized = get_stake_state(destination_stake_account_info)? {
        // we can split into this
    } else {
        return Err(ProgramError::InvalidAccountData);
    }

    let source_lamport_balance = source_stake_account_info.lamports();
    let destination_lamport_balance = destination_stake_account_info.lamports();

    if split_lamports > source_lamport_balance {
        return Err(ProgramError::InsufficientFunds);
    }

    match get_stake_state(source_stake_account_info)? {
        StakeStateV2::Stake(source_meta, mut source_stake, stake_flags) => {
            source_meta
                .authorized
                .check(&signers, StakeAuthorize::Staker)
                .map_err(to_program_error)?;

            let minimum_delegation = crate::get_minimum_delegation();

            let status = source_stake.delegation.stake_activating_and_deactivating(
                clock.epoch,
                stake_history,
                PERPETUAL_NEW_WARMUP_COOLDOWN_RATE_EPOCH,
            );

            let is_active = status.effective > 0;

            // NOTE this function also internally summons Rent via syscall
            let validated_split_info = validate_split_amount(
                source_lamport_balance,
                destination_lamport_balance,
                split_lamports,
                &source_meta,
                destination_data_len,
                minimum_delegation,
                is_active,
            )?;

            // split the stake, subtract rent_exempt_balance unless
            // the destination account already has those lamports
            // in place.
            // this means that the new stake account will have a stake equivalent to
            // lamports minus rent_exempt_reserve if it starts out with a zero balance
            let (remaining_stake_delta, split_stake_amount) =
                if validated_split_info.source_remaining_balance == 0 {
                    // If split amount equals the full source stake (as implied by 0
                    // source_remaining_balance), the new split stake must equal the same
                    // amount, regardless of any current lamport balance in the split account.
                    // Since split accounts retain the state of their source account, this
                    // prevents any magic activation of stake by prefunding the split account.
                    //
                    // The new split stake also needs to ignore any positive delta between the
                    // original rent_exempt_reserve and the split_rent_exempt_reserve, in order
                    // to prevent magic activation of stake by splitting between accounts of
                    // different sizes.
                    let remaining_stake_delta =
                        split_lamports.saturating_sub(source_meta.rent_exempt_reserve);
                    (remaining_stake_delta, remaining_stake_delta)
                } else {
                    // Otherwise, the new split stake should reflect the entire split
                    // requested, less any lamports needed to cover the
                    // split_rent_exempt_reserve.
                    if source_stake.delegation.stake.saturating_sub(split_lamports)
                        < minimum_delegation
                    {
                        return Err(StakeError::InsufficientDelegation.into());
                    }

                    (
                        split_lamports,
                        split_lamports.saturating_sub(
                            validated_split_info
                                .destination_rent_exempt_reserve
                                .saturating_sub(destination_lamport_balance),
                        ),
                    )
                };

            if split_stake_amount < minimum_delegation {
                return Err(StakeError::InsufficientDelegation.into());
            }

            let destination_stake =
                source_stake.split(remaining_stake_delta, split_stake_amount)?;

            let mut destination_meta = source_meta;
            destination_meta.rent_exempt_reserve =
                validated_split_info.destination_rent_exempt_reserve;

            set_stake_state(
                source_stake_account_info,
                &StakeStateV2::Stake(source_meta, source_stake, stake_flags),
            )?;

            set_stake_state(
                destination_stake_account_info,
                &StakeStateV2::Stake(destination_meta, destination_stake, stake_flags),
            )?;
        }
        StakeStateV2::Initialized(source_meta) => {
            source_meta
                .authorized
                .check(&signers, StakeAuthorize::Staker)
                .map_err(to_program_error)?;

            // NOTE this function also internally summons Rent via syscall
            let validated_split_info = validate_split_amount(
                source_lamport_balance,
                destination_lamport_balance,
                split_lamports,
                &source_meta,
                destination_data_len,
                0,     // additional_required_lamports
                false, // is_active
            )?;

            let mut destination_meta = source_meta;
            destination_meta.rent_exempt_reserve =
                validated_split_info.destination_rent_exempt_reserve;

            set_stake_state(
                destination_stake_account_info,
                &StakeStateV2::Initialized(destination_meta),
            )?;
        }
        StakeStateV2::Uninitialized => {
            if !source_stake_account_info.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }
        }
        _ => return Err(ProgramError::InvalidAccountData),
    }

    // Deinitialize state upon zero balance
    if split_lamports == source_lamport_balance {
        set_stake_state(source_stake_account_info, &StakeStateV2::Uninitialized)?;
    }

    relocate_lamports(
        source_stake_account_info,
        destination_stake_account_info,
        split_lamports,
    )?;

    Ok(())
}
