//! Account state and shared types for the quadratic-voting program.

use anchor_lang::prelude::*;

use crate::errors::QuadraticVotingError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum VoteChoice {
    Yes,
    No,
    Abstain,
}

#[account]
pub struct QuadraticBallot {
    pub authority: Pubkey,
    pub realm: Pubkey,
    pub proposal: Pubkey,
    pub bump: u8,
    pub min_reputation_bps: u16,
    pub max_reputation_bps: u16,
    pub voting_starts_at: i64,
    pub voting_ends_at: i64,
    pub finalized: bool,
    pub total_registered_voters: u32,
    pub total_credits_budget: u64,
    pub yes_tally_scaled: u128,
    pub no_tally_scaled: u128,
    pub abstain_tally_scaled: u128,
}

impl QuadraticBallot {
    pub const LEN: usize =
        32 + 32 + 32 + 1 + 2 + 2 + 8 + 8 + 1 + 4 + 8 + 16 + 16 + 16;

    pub fn add_tally(&mut self, choice: VoteChoice, weighted_increment_scaled: u128) -> Result<()> {
        match choice {
            VoteChoice::Yes => {
                self.yes_tally_scaled = self
                    .yes_tally_scaled
                    .checked_add(weighted_increment_scaled)
                    .ok_or(QuadraticVotingError::MathOverflow)?;
            }
            VoteChoice::No => {
                self.no_tally_scaled = self
                    .no_tally_scaled
                    .checked_add(weighted_increment_scaled)
                    .ok_or(QuadraticVotingError::MathOverflow)?;
            }
            VoteChoice::Abstain => {
                self.abstain_tally_scaled = self
                    .abstain_tally_scaled
                    .checked_add(weighted_increment_scaled)
                    .ok_or(QuadraticVotingError::MathOverflow)?;
            }
        }
        Ok(())
    }
}

#[account]
pub struct VoterAllocation {
    pub ballot: Pubkey,
    pub voter: Pubkey,
    pub bump: u8,
    pub reputation_multiplier_bps: u16,
    pub credits_budget: u64,
    pub credits_spent: u64,
    pub yes_votes: u32,
    pub no_votes: u32,
    pub abstain_votes: u32,
    pub last_updated_slot: u64,
}

impl VoterAllocation {
    pub const LEN: usize = 32 + 32 + 1 + 2 + 8 + 8 + 4 + 4 + 4 + 8;

    pub fn votes_for_choice(&self, choice: VoteChoice) -> u32 {
        match choice {
            VoteChoice::Yes => self.yes_votes,
            VoteChoice::No => self.no_votes,
            VoteChoice::Abstain => self.abstain_votes,
        }
    }

    pub fn set_votes_for_choice(&mut self, choice: VoteChoice, value: u32) {
        match choice {
            VoteChoice::Yes => self.yes_votes = value,
            VoteChoice::No => self.no_votes = value,
            VoteChoice::Abstain => self.abstain_votes = value,
        }
    }
}
