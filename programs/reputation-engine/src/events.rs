//! Events emitted by the reputation-engine program.

use borsh::BorshSerialize;
use solana_program::{log::sol_log_data, pubkey::Pubkey};

#[derive(BorshSerialize)]
pub struct RealmConfigInitializedEvent {
    pub realm: Pubkey,
    pub admin: Pubkey,
    pub oracle_authority: Pubkey,
}

#[derive(BorshSerialize)]
pub struct ProfileCreatedEvent {
    pub realm: Pubkey,
    pub member: Pubkey,
    pub multiplier_bps: u16,
}

#[derive(BorshSerialize)]
pub struct ProfileUpdatedEvent {
    pub realm: Pubkey,
    pub member: Pubkey,
    pub multiplier_bps: u16,
    pub penalties_score: u32,
}

#[derive(BorshSerialize)]
pub struct PenaltyAppliedEvent {
    pub realm: Pubkey,
    pub penalty_points: u32,
    pub reason_code: u16,
    pub member: Pubkey,
    pub multiplier_bps: u16,
}

#[derive(BorshSerialize)]
pub struct MultiplierSnapshotEvent {
    pub realm: Pubkey,
    pub member: Pubkey,
    pub multiplier_bps: u16,
    pub slot: u64,
}

pub fn emit_event<T: BorshSerialize>(event: &T) {
    let data = borsh::to_vec(event).unwrap();
    sol_log_data(&[&data]);
}
