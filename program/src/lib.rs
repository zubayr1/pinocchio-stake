#![no_std]

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

#[cfg(feature = "std")]
extern crate std;

pub mod consts;
pub mod error;
pub mod helpers;
pub mod instruction;
pub mod state;

pub use pinocchio::pubkey::Pubkey;

pinocchio_pubkey::declare_id!("4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT");
