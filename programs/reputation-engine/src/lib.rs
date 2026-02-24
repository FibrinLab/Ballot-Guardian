//! Reputation engine program: behavior-based scoring and bounded multiplier computation.
//!
//! Layout is modular: state, events, errors, and helpers live in separate modules.

use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};

pub mod error;
pub mod events;
pub mod helpers;
pub mod instruction;
pub mod processor;
pub mod state;

pub use error::*;
pub use events::*;
pub use instruction::*;
pub use state::*;

// TODO: Replace with deployed program ID
declare_id!("11111111111111111111111111111111");

#[cfg(not(feature = "no-entrypoint"))]
solana_program::entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = instruction::ReputationInstruction::try_from_slice(instruction_data)
        .map_err(|_| solana_program::program_error::ProgramError::InvalidInstructionData)?;

    match ix {
        ReputationInstruction::InitializeRealmConfig(args) => {
            msg!("Instruction: InitializeRealmConfig");
            processor::process_initialize_realm_config(program_id, accounts, args)
        }
        ReputationInstruction::SetOracleAuthority {
            new_oracle_authority,
        } => {
            msg!("Instruction: SetOracleAuthority");
            processor::process_set_oracle_authority(program_id, accounts, new_oracle_authority)
        }
        ReputationInstruction::CreateProfile => {
            msg!("Instruction: CreateProfile");
            processor::process_create_profile(program_id, accounts)
        }
        ReputationInstruction::ApplyComponentDelta(args) => {
            msg!("Instruction: ApplyComponentDelta");
            processor::process_apply_component_delta(program_id, accounts, args)
        }
        ReputationInstruction::ApplyPenalty {
            penalty_points,
            reason_code,
        } => {
            msg!("Instruction: ApplyPenalty");
            processor::process_apply_penalty(program_id, accounts, penalty_points, reason_code)
        }
        ReputationInstruction::RecalculateProfile => {
            msg!("Instruction: RecalculateProfile");
            processor::process_recalculate_profile(program_id, accounts)
        }
        ReputationInstruction::SnapshotMultiplier => {
            msg!("Instruction: SnapshotMultiplier");
            processor::process_snapshot_multiplier(program_id, accounts)
        }
    }
}
