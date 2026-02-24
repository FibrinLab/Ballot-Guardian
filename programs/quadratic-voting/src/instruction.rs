//! Instruction definitions for the quadratic-voting program.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::state::VoteChoice;

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum QuadraticVotingInstruction {
    /// Initialize a quadratic voting ballot.
    ///
    /// Accounts:
    /// 0. `[signer, writable]` authority (payer)
    /// 1. `[writable]` ballot PDA (seeds: ["ballot", realm, proposal])
    /// 2. `[]` system_program
    InitializeBallot(InitializeBallotArgs),

    /// Register a voter on a ballot with a credit budget.
    ///
    /// Accounts:
    /// 0. `[signer, writable]` authority (payer)
    /// 1. `[writable]` ballot PDA
    /// 2. `[]` voter
    /// 3. `[writable]` allocation PDA (seeds: ["allocation", ballot, voter])
    /// 4. `[]` system_program
    RegisterVoter(RegisterVoterArgs),

    /// Update the reputation snapshot for a voter before voting starts.
    ///
    /// Accounts:
    /// 0. `[signer]` authority
    /// 1. `[]` ballot PDA
    /// 2. `[]` voter
    /// 3. `[writable]` allocation PDA
    UpdateVoterReputationSnapshot {
        new_reputation_multiplier_bps: u16,
    },

    /// Cast a vote (quadratic cost deducted from credit budget).
    ///
    /// Accounts:
    /// 0. `[signer]` voter
    /// 1. `[writable]` ballot PDA
    /// 2. `[writable]` allocation PDA
    CastVote {
        choice: VoteChoice,
        additional_votes: u32,
    },

    /// Finalize a ballot after voting ends.
    ///
    /// Accounts:
    /// 0. `[signer]` authority
    /// 1. `[writable]` ballot PDA
    FinalizeBallot,
}

#[derive(Debug, Clone, Copy, BorshSerialize, BorshDeserialize)]
pub struct InitializeBallotArgs {
    pub realm: Pubkey,
    pub proposal: Pubkey,
    pub min_reputation_bps: u16,
    pub max_reputation_bps: u16,
    pub voting_starts_at: i64,
    pub voting_ends_at: i64,
}

#[derive(Debug, Clone, Copy, BorshSerialize, BorshDeserialize)]
pub struct RegisterVoterArgs {
    pub credits_budget: u64,
    pub reputation_multiplier_bps: u16,
}
