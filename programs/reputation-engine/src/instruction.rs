//! Instruction definitions for the reputation-engine program.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum ReputationInstruction {
    /// Initialize a realm reputation configuration.
    ///
    /// Accounts:
    /// 0. `[signer, writable]` admin (payer)
    /// 1. `[writable]` config PDA (seeds: ["reputation-config", realm])
    /// 2. `[]` system_program
    InitializeRealmConfig(InitializeRealmConfigArgs),

    /// Set a new oracle authority on an existing config.
    ///
    /// Accounts:
    /// 0. `[signer]` admin
    /// 1. `[writable]` config PDA
    SetOracleAuthority { new_oracle_authority: Pubkey },

    /// Create a reputation profile for a member.
    ///
    /// Accounts:
    /// 0. `[signer, writable]` payer
    /// 1. `[]` config PDA
    /// 2. `[]` member
    /// 3. `[writable]` profile PDA (seeds: ["reputation-profile", realm, member])
    /// 4. `[]` system_program
    CreateProfile,

    /// Apply component score deltas to a profile.
    ///
    /// Accounts:
    /// 0. `[signer]` updater (admin or oracle)
    /// 1. `[]` config PDA
    /// 2. `[]` member
    /// 3. `[writable]` profile PDA
    ApplyComponentDelta(ApplyComponentDeltaArgs),

    /// Apply a penalty to a profile.
    ///
    /// Accounts:
    /// 0. `[signer]` updater (admin or oracle)
    /// 1. `[]` config PDA
    /// 2. `[]` member
    /// 3. `[writable]` profile PDA
    ApplyPenalty {
        penalty_points: u32,
        reason_code: u16,
    },

    /// Recalculate the multiplier for a profile.
    ///
    /// Accounts:
    /// 0. `[signer]` updater (admin or oracle)
    /// 1. `[]` config PDA
    /// 2. `[]` member
    /// 3. `[writable]` profile PDA
    RecalculateProfile,

    /// Snapshot the current multiplier as an event (read-only).
    ///
    /// Accounts:
    /// 0. `[]` config PDA
    /// 1. `[]` member
    /// 2. `[]` profile PDA
    SnapshotMultiplier,
}

#[derive(Debug, Clone, Copy, BorshSerialize, BorshDeserialize)]
pub struct InitializeRealmConfigArgs {
    pub realm: Pubkey,
    pub oracle_authority: Option<Pubkey>,
    pub min_multiplier_bps: u16,
    pub base_multiplier_bps: u16,
    pub max_multiplier_bps: u16,
    pub participation_weight: u16,
    pub proposal_weight: u16,
    pub staking_weight: u16,
    pub tenure_weight: u16,
    pub delegation_weight: u16,
    pub points_per_bonus_bps: u32,
    pub penalty_unit_bps: u16,
    pub max_bonus_bps: u16,
    pub max_penalty_bps: u16,
}

#[derive(Debug, Clone, Copy, Default, BorshSerialize, BorshDeserialize)]
pub struct ApplyComponentDeltaArgs {
    pub participation_delta: i32,
    pub proposal_delta: i32,
    pub staking_delta: i32,
    pub tenure_delta: i32,
    pub delegation_delta: i32,
}
