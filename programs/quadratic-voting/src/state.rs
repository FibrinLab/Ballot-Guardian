//! Account state and shared types for the quadratic-voting program.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::error::QuadraticVotingError;
use solana_program::program_error::ProgramError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
#[borsh(use_discriminant = true)]
#[repr(u8)]
pub enum AccountTag {
    Uninitialized = 0,
    QuadraticBallot = 1,
    VoterAllocation = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
#[borsh(use_discriminant = true)]
#[repr(u8)]
pub enum VoteChoice {
    Yes = 0,
    No = 1,
    Abstain = 2,
}

#[derive(Debug, Clone, Copy, BorshSerialize, BorshDeserialize)]
pub struct QuadraticBallot {
    pub tag: u8,
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
        1 + 32 + 32 + 32 + 1 + 2 + 2 + 8 + 8 + 1 + 4 + 8 + 16 + 16 + 16;

    pub fn add_tally(
        &mut self,
        choice: VoteChoice,
        weighted_increment_scaled: u128,
    ) -> Result<(), ProgramError> {
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

#[derive(Debug, Clone, Copy, BorshSerialize, BorshDeserialize)]
pub struct VoterAllocation {
    pub tag: u8,
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
    pub const LEN: usize = 1 + 32 + 32 + 1 + 2 + 8 + 8 + 4 + 4 + 4 + 8;

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
