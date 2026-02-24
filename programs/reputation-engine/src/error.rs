//! Error types for the reputation-engine program.

use solana_program::program_error::ProgramError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ReputationError {
    Unauthorized = 0,
    InvalidMultiplierConfig = 1,
    InvalidScoringConfig = 2,
    MathOverflow = 3,
    InvalidPenalty = 4,
    ProfileRealmMismatch = 5,
    ProfileMemberMismatch = 6,
    InvalidAccountTag = 7,
    InvalidAccountOwner = 8,
}

impl From<ReputationError> for ProgramError {
    fn from(e: ReputationError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
