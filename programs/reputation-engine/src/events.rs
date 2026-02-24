//! Events emitted by the reputation-engine program.

use anchor_lang::prelude::*;

#[event]
pub struct RealmConfigInitializedEvent {
    pub realm: Pubkey,
    pub admin: Pubkey,
    pub oracle_authority: Pubkey,
}

#[event]
pub struct ProfileCreatedEvent {
    pub realm: Pubkey,
    pub member: Pubkey,
    pub multiplier_bps: u16,
}

#[event]
pub struct ProfileUpdatedEvent {
    pub realm: Pubkey,
    pub member: Pubkey,
    pub multiplier_bps: u16,
    pub penalties_score: u32,
}

#[event]
pub struct PenaltyAppliedEvent {
    pub realm: Pubkey,
    pub member: Pubkey,
    pub penalty_points: u32,
    pub reason_code: u16,
    pub multiplier_bps: u16,
}

#[event]
pub struct MultiplierSnapshotEvent {
    pub realm: Pubkey,
    pub member: Pubkey,
    pub multiplier_bps: u16,
    pub slot: u64,
}
