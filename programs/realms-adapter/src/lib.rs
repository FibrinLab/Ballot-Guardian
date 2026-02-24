//! Realms adapter program: binds Realms proposals to quadratic ballots and publishes voter weight records.
//!
//! Layout is modular: state, events, errors, and math live in separate modules.

use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};

pub mod error;
pub mod events;
pub mod instruction;
pub mod math;
pub mod processor;
pub mod state;

pub use error::*;
pub use events::*;
pub use instruction::*;
pub use state::*;

declare_id!("E5CHyQY6gsxWB4cdTCSMxS3aY3J4eCVCXEe1KVTfk4Ky");

#[cfg(not(feature = "no-entrypoint"))]
solana_program::entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = instruction::RealmsAdapterInstruction::try_from_slice(instruction_data)
        .map_err(|_| solana_program::program_error::ProgramError::InvalidInstructionData)?;

    match ix {
        RealmsAdapterInstruction::InitializeAdapter(args) => {
            msg!("Instruction: InitializeAdapter");
            processor::process_initialize_adapter(program_id, accounts, args)
        }
        RealmsAdapterInstruction::BindProposal(args) => {
            msg!("Instruction: BindProposal");
            processor::process_bind_proposal(program_id, accounts, args)
        }
        RealmsAdapterInstruction::SetCouncilOverride {
            active,
            reason_code,
        } => {
            msg!("Instruction: SetCouncilOverride");
            processor::process_set_council_override(
                program_id,
                accounts,
                active,
                reason_code,
            )
        }
        RealmsAdapterInstruction::CreateVoterWeightRecord => {
            msg!("Instruction: CreateVoterWeightRecord");
            processor::process_create_voter_weight_record(program_id, accounts)
        }
        RealmsAdapterInstruction::RefreshVoterWeightRecord(args) => {
            msg!("Instruction: RefreshVoterWeightRecord");
            processor::process_refresh_voter_weight_record(program_id, accounts, args)
        }
    }
}
