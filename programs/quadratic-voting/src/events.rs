//! Events emitted by the quadratic-voting program.

use borsh::BorshSerialize;
use solana_program::{log::sol_log_data, pubkey::Pubkey};

use crate::state::VoteChoice;

#[derive(BorshSerialize)]
pub struct BallotInitializedEvent {
    pub ballot: Pubkey,
    pub realm: Pubkey,
    pub proposal: Pubkey,
    pub authority: Pubkey,
    pub voting_starts_at: i64,
    pub voting_ends_at: i64,
}

#[derive(BorshSerialize)]
pub struct VoterRegisteredEvent {
    pub ballot: Pubkey,
    pub voter: Pubkey,
    pub credits_budget: u64,
    pub reputation_multiplier_bps: u16,
}

#[derive(BorshSerialize)]
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

#[derive(BorshSerialize)]
pub struct BallotFinalizedEvent {
    pub ballot: Pubkey,
    pub yes_tally_scaled: u128,
    pub no_tally_scaled: u128,
    pub abstain_tally_scaled: u128,
}

pub fn emit_event<T: BorshSerialize>(event: &T) {
    let data = borsh::to_vec(event).unwrap();
    sol_log_data(&[&data]);
}
