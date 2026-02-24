//! Events emitted by the realms-adapter program.

use borsh::BorshSerialize;
use solana_program::{log::sol_log_data, pubkey::Pubkey};

#[derive(BorshSerialize)]
pub struct AdapterInitializedEvent {
    pub realm: Pubkey,
    pub admin: Pubkey,
}

#[derive(BorshSerialize)]
pub struct ProposalBoundEvent {
    pub realm: Pubkey,
    pub proposal: Pubkey,
    pub quadratic_ballot: Pubkey,
}

#[derive(BorshSerialize)]
pub struct CouncilOverrideUpdatedEvent {
    pub proposal: Pubkey,
    pub active: bool,
    pub reason_code: u16,
}

#[derive(BorshSerialize)]
pub struct VoterWeightRecordRefreshedEvent {
    pub proposal: Pubkey,
    pub voter: Pubkey,
    pub voter_weight: u64,
    pub qv_component: u64,
    pub reputation_multiplier_bps: u16,
    pub council_override_active: bool,
}

pub fn emit_event<T: BorshSerialize>(event: &T) {
    let data = borsh::to_vec(event).unwrap();
    sol_log_data(&[&data]);
}
