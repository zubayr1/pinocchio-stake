#![allow(unexpected_cfgs)]

use crate::instruction::{self, StakeInstruction};
use pinocchio::{
    account_info::AccountInfo, default_panic_handler, no_allocator, program_entrypoint,
    program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

// This is the entrypoint for the program.
program_entrypoint!(process_instruction);
//Do not allocate memory.
no_allocator!();
// Use the no_std panic handler.
default_panic_handler!();

#[inline(always)]
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // convenience so we can safely use id() everywhere
    if *program_id != crate::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    let (ix_disc, instruction_data) = instruction_data
        .split_first_chunk::<4>()
        .ok_or(ProgramError::InvalidInstructionData)?;

    // Second variant, test CUs usage
    // let (ix_disc, instruction_data) = instruction_data
    //     .split_at_checked(4)
    //     .ok_or(ProgramError::InvalidInstructionData)?;

    let instruction = StakeInstruction::try_from(&ix_disc[0])?;

    // TODO: add check for epoch_rewards_active
    // let epoch_rewards_active = EpochRewards::get()
    //         .map(|epoch_rewards| epoch_rewards.active)
    //         .unwrap_or(false);
    // if epoch_rewards_active && !matches!(instruction, StakeInstruction::GetMinimumDelegation) {
    //     return Err(StakeError::EpochRewardsActive.into());
    // }

    match instruction {
        StakeInstruction::Initialize => {
            #[cfg(feature = "logging")]
            pinocchio_log::log!("Instruction: Initialize");

            //TODO
        }
        StakeInstruction::Authorize => {
            #[cfg(feature = "logging")]
            pinocchio_log::log!("Instruction: Authorize");
            //TODO
        }
        StakeInstruction::DelegateStake => {
            #[cfg(feature = "logging")]
            pinocchio_log::log!("Instruction: DelegateStake");
            //TODO
        }
        StakeInstruction::Split => {
            #[cfg(feature = "logging")]
            pinocchio_log::log!("Instruction: Split");
            //TODO
        }
        StakeInstruction::Withdraw => {
            #[cfg(feature = "logging")]
            pinocchio_log::log!("Instruction: Withdraw");
            //TODO
        }
        StakeInstruction::Deactivate => {
            #[cfg(feature = "logging")]
            pinocchio_log::log!("Instruction: Deactivate");
            //TODO
        }
        StakeInstruction::SetLockup => {
            #[cfg(feature = "logging")]
            pinocchio_log::log!("Instruction: SetLockup");
            //TODO
        }
        StakeInstruction::Merge => {
            #[cfg(feature = "logging")]
            pinocchio_log::log!("Instruction: Merge");
            //TODO
        }
        StakeInstruction::AuthorizeWithSeed => {
            #[cfg(feature = "logging")]
            pinocchio_log::log!("Instruction: AuthorizeWithSeed");
            //TODO
        }
        StakeInstruction::InitializeChecked => {
            #[cfg(feature = "logging")]
            pinocchio_log::log!("Instruction: InitializeChecked");
            //TODO
        }
        StakeInstruction::AuthorizeChecked => {
            #[cfg(feature = "logging")]
            pinocchio_log::log!("Instruction: InitializeChecked");
            //TODO
        }
        StakeInstruction::AuthorizeCheckedWithSeed => {
            #[cfg(feature = "logging")]
            pinocchio_log::log!("Instruction: AuthorizeCheckedWithSeed");
            //TODO
        }
        StakeInstruction::SetLockupChecked => {
            #[cfg(feature = "logging")]
            pinocchio_log::log!("Instruction: SetLockupChecked");
            //TODO
        }
        StakeInstruction::GetMinimumDelegation => {
            #[cfg(feature = "logging")]
            pinocchio_log::log!("Instruction: GetMinimumDelegation");
            //TODO
        }
        StakeInstruction::DeactivateDelinquent => {
            #[cfg(feature = "logging")]
            pinocchio_log::log!("Instruction: DeactivateDelinquent");
            //TODO
        }
        #[allow(deprecated)]
        StakeInstruction::Redelegate => {
            //Err(ProgramError::InvalidInstructionData),
            // NOTE we assume the program is going live after `move_stake_and_move_lamports_ixs` is
            // activated
        }
        StakeInstruction::MoveStake => {
            #[cfg(feature = "logging")]
            pinocchio_log::log!("Instruction: MoveStake");
            //TODO
        }
        StakeInstruction::MoveLamports => {
            #[cfg(feature = "logging")]
            pinocchio_log::log!("Instruction: MoveLamports");
            //TODO
        }
    }
    Ok(())
}
