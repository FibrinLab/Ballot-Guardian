//! Events emitted by the realms-adapter program.

use anchor_lang::prelude::*;

#[event]
pub struct AdapterInitializedEvent {
    pub realm: Pubkey,
    pub admin: Pubkey,
}

#[event]
pub struct ProposalBoundEvent {
    pub realm: Pubkey,
    pub proposal: Pubkey,
    pub quadratic_ballot: Pubkey,
}

#[event]
pub struct CouncilOverrideUpdatedEvent {
    pub proposal: Pubkey,
    pub active: bool,
    pub reason_code: u16,
}

#[event]
pub struct VoterWeightRecordRefreshedEvent {
    pub proposal: Pubkey,
    pub voter: Pubkey,
    pub voter_weight: u64,
    pub qv_component: u64,
    pub reputation_multiplier_bps: u16,
    pub council_override_active: bool,
}
