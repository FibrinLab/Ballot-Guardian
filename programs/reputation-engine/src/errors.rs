//! Error types for the reputation-engine program.

use anchor_lang::prelude::*;

#[error_code]
pub enum ReputationError {
    #[msg("Unauthorized updater")]
    Unauthorized,
    #[msg("Invalid multiplier configuration")]
    InvalidMultiplierConfig,
    #[msg("Invalid scoring configuration")]
    InvalidScoringConfig,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Invalid penalty input")]
    InvalidPenalty,
    #[msg("Profile does not match realm")]
    ProfileRealmMismatch,
    #[msg("Profile does not match member")]
    ProfileMemberMismatch,
}
