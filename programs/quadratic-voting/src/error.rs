//! Error types for the quadratic-voting program.

use solana_program::program_error::ProgramError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QuadraticVotingError {
    Unauthorized = 0,
    InvalidVotingWindow = 1,
    InvalidReputationBounds = 2,
    InvalidCreditsBudget = 3,
    VotingNotStarted = 4,
    VotingWindowClosed = 5,
    VotingStillActive = 6,
    BallotFinalized = 7,
    MathOverflow = 8,
    CreditBudgetExceeded = 9,
    ZeroAdditionalVotes = 10,
    ReputationMultiplierOutOfBounds = 11,
    VotingAlreadyStarted = 12,
    CannotChangeMultiplierAfterVoting = 13,
    AllocationBallotMismatch = 14,
    AllocationVoterMismatch = 15,
    InvalidAccountTag = 16,
    InvalidAccountOwner = 17,
}

impl From<QuadraticVotingError> for ProgramError {
    fn from(e: QuadraticVotingError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
