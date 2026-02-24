//! Error types for the realms-adapter program.

use anchor_lang::prelude::*;

#[error_code]
pub enum RealmsAdapterError {
    #[msg("Unauthorized caller")]
    Unauthorized,
    #[msg("Invalid reputation bounds")]
    InvalidReputationBounds,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Binding does not belong to config")]
    BindingConfigMismatch,
    #[msg("Binding does not match proposal")]
    BindingProposalMismatch,
    #[msg("Weight record does not belong to binding")]
    WeightRecordBindingMismatch,
    #[msg("Weight record does not belong to voter")]
    WeightRecordVoterMismatch,
    #[msg("Governing token mint mismatch")]
    MintMismatch,
    #[msg("Council override is disabled for this proposal")]
    CouncilOverrideDisabled,
}
