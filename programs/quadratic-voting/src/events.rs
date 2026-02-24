//! Events emitted by the quadratic-voting program.

use anchor_lang::prelude::*;

use crate::state::VoteChoice;

#[event]
pub struct BallotInitializedEvent {
    pub ballot: Pubkey,
    pub realm: Pubkey,
    pub proposal: Pubkey,
    pub authority: Pubkey,
    pub voting_starts_at: i64,
    pub voting_ends_at: i64,
}

#[event]
pub struct VoterRegisteredEvent {
    pub ballot: Pubkey,
    pub voter: Pubkey,
    pub credits_budget: u64,
    pub reputation_multiplier_bps: u16,
}

#[event]
pub struct VoteCastEvent {
    pub ballot: Pubkey,
    pub voter: Pubkey,
    pub choice: VoteChoice,
    pub added_votes: u32,
    pub incremental_cost: u64,
    pub credits_spent: u64,
    pub reputation_multiplier_bps: u16,
    pub weighted_increment_scaled: u128,
}

#[event]
pub struct BallotFinalizedEvent {
    pub ballot: Pubkey,
    pub yes_tally_scaled: u128,
    pub no_tally_scaled: u128,
    pub abstain_tally_scaled: u128,
}
