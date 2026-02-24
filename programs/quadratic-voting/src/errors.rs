//! Error types for the quadratic-voting program.

use anchor_lang::prelude::*;

#[error_code]
pub enum QuadraticVotingError {
    #[msg("Unauthorized caller")]
    Unauthorized,
    #[msg("Invalid voting window")]
    InvalidVotingWindow,
    #[msg("Invalid reputation multiplier bounds")]
    InvalidReputationBounds,
    #[msg("Invalid credit budget")]
    InvalidCreditsBudget,
    #[msg("Voting has not started")]
    VotingNotStarted,
    #[msg("Voting window is closed")]
    VotingWindowClosed,
    #[msg("Voting is still active")]
    VotingStillActive,
    #[msg("Ballot already finalized")]
    BallotFinalized,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Credit budget exceeded")]
    CreditBudgetExceeded,
    #[msg("Zero votes requested")]
    ZeroAdditionalVotes,
    #[msg("Reputation multiplier outside ballot bounds")]
    ReputationMultiplierOutOfBounds,
    #[msg("Voting already started")]
    VotingAlreadyStarted,
    #[msg("Cannot change multiplier after voting activity")]
    CannotChangeMultiplierAfterVoting,
    #[msg("Voter allocation belongs to a different ballot")]
    AllocationBallotMismatch,
    #[msg("Voter allocation belongs to a different voter")]
    AllocationVoterMismatch,
}
