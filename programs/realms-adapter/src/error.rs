//! Error types for the realms-adapter program.

use solana_program::program_error::ProgramError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum RealmsAdapterError {
    Unauthorized = 0,
    InvalidReputationBounds = 1,
    MathOverflow = 2,
    BindingConfigMismatch = 3,
    BindingProposalMismatch = 4,
    WeightRecordBindingMismatch = 5,
    WeightRecordVoterMismatch = 6,
    MintMismatch = 7,
    CouncilOverrideDisabled = 8,
    InvalidDiscriminator = 9,
    InvalidAccountOwner = 10,
}

impl From<RealmsAdapterError> for ProgramError {
    fn from(e: RealmsAdapterError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
