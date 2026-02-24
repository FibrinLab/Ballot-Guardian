//! Account state for the reputation-engine program.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
#[borsh(use_discriminant = true)]
#[repr(u8)]
pub enum AccountTag {
    Uninitialized = 0,
    RealmReputationConfig = 1,
    ReputationProfile = 2,
}

#[derive(Debug, Clone, Copy, BorshSerialize, BorshDeserialize)]
pub struct RealmReputationConfig {
    pub tag: u8,
    pub realm: Pubkey,
    pub admin: Pubkey,
    pub oracle_authority: Pubkey,
    pub bump: u8,
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

impl RealmReputationConfig {
    pub const LEN: usize = 1 + 32 + 32 + 32 + 1 + (2 * 11) + 4;
}

#[derive(Debug, Clone, Copy, BorshSerialize, BorshDeserialize)]
pub struct ReputationProfile {
    pub tag: u8,
    pub realm: Pubkey,
    pub member: Pubkey,
    pub bump: u8,
    pub participation_score: u32,
    pub proposal_creation_score: u32,
    pub staking_score: u32,
    pub tenure_score: u32,
    pub delegation_trust_score: u32,
    pub penalties_score: u32,
    pub multiplier_bps: u16,
    pub last_updated_slot: u64,
}

impl ReputationProfile {
    pub const LEN: usize = 1 + 32 + 32 + 1 + (4 * 6) + 2 + 8;
}
