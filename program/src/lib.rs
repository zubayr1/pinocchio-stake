#![cfg_attr(not(test), no_std)]

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

#[cfg(feature = "std")]
extern crate std;

pub mod consts;
pub mod error;
pub mod instruction;
pub mod state;

pinocchio_pubkey::declare_id!("Stake11111111111111111111111111111111111111");
