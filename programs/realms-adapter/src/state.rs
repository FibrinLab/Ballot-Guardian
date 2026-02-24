//! Account state for the realms-adapter program.

use anchor_lang::prelude::*;

#[account]
pub struct AdapterConfig {
    pub realm: Pubkey,
    pub admin: Pubkey,
    pub governance_program_id: Pubkey,
    pub quadratic_voting_program: Pubkey,
    pub reputation_engine_program: Pubkey,
    pub council_override_authority: Pubkey,
    pub bump: u8,
    pub min_reputation_bps: u16,
    pub max_reputation_bps: u16,
}

impl AdapterConfig {
    pub const LEN: usize = (32 * 6) + 1 + 2 + 2;
}

#[account]
pub struct ProposalBinding {
    pub adapter_config: Pubkey,
    pub realm: Pubkey,
    pub proposal: Pubkey,
    pub quadratic_ballot: Pubkey,
    pub governing_token_mint: Pubkey,
    pub bump: u8,
    pub council_override_enabled: bool,
    pub council_override_active: bool,
    pub council_override_reason_code: u16,
    pub default_weight_expiry_slot: u64,
    pub last_weight_refresh_slot: u64,
}

impl ProposalBinding {
    pub const LEN: usize = (32 * 5) + 1 + 1 + 1 + 2 + 8 + 8;
}

#[account]
pub struct PluginVoterWeightRecord {
    pub binding: Pubkey,
    pub voter: Pubkey,
    pub governing_token_mint: Pubkey,
    pub bump: u8,
    pub voter_weight: u64,
    pub voter_weight_expiry_slot: Option<u64>,
    pub weight_action_target: Option<Pubkey>,
    pub token_amount_allocated: u64,
    pub qv_votes_allocated: u32,
    pub reputation_multiplier_bps: u16,
    pub last_updated_slot: u64,
    pub council_override_active: bool,
}

impl PluginVoterWeightRecord {
    pub const LEN: usize =
        (32 * 3) + 1 + 8 + (1 + 8) + (1 + 32) + 8 + 4 + 2 + 8 + 1;
}
