//! Quadratic voting program: proposal-level ballots, credit budgets, and reputation-scaled tallies.
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
pub use math::*;
pub use state::*;

declare_id!("346RNEQcBBff4skHhQiRCPe9cDeWpaPTsT2TpQUFYomp");

#[cfg(not(feature = "no-entrypoint"))]
solana_program::entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = instruction::QuadraticVotingInstruction::try_from_slice(instruction_data)
        .map_err(|_| solana_program::program_error::ProgramError::InvalidInstructionData)?;

    match ix {
        QuadraticVotingInstruction::InitializeBallot(args) => {
            msg!("Instruction: InitializeBallot");
            processor::process_initialize_ballot(program_id, accounts, args)
        }
        QuadraticVotingInstruction::RegisterVoter(args) => {
            msg!("Instruction: RegisterVoter");
            processor::process_register_voter(program_id, accounts, args)
        }
        QuadraticVotingInstruction::UpdateVoterReputationSnapshot {
            new_reputation_multiplier_bps,
        } => {
            msg!("Instruction: UpdateVoterReputationSnapshot");
            processor::process_update_voter_reputation_snapshot(
                program_id,
                accounts,
                new_reputation_multiplier_bps,
            )
        }
        QuadraticVotingInstruction::CastVote {
            choice,
            additional_votes,
        } => {
            msg!("Instruction: CastVote");
            processor::process_cast_vote(program_id, accounts, choice, additional_votes)
        }
        QuadraticVotingInstruction::FinalizeBallot => {
            msg!("Instruction: FinalizeBallot");
            processor::process_finalize_ballot(program_id, accounts)
        }
    }
}
