//! Instruction definitions for the realms-adapter program.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::state::VoterWeightAction;

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum RealmsAdapterInstruction {
    /// Initialize the adapter configuration for a realm.
    ///
    /// Accounts:
    /// 0. `[signer, writable]` admin (payer)
    /// 1. `[writable]` config PDA (seeds: ["adapter-config", realm])
    /// 2. `[]` system_program
    InitializeAdapter(InitializeAdapterArgs),

    /// Bind a Realms proposal to a quadratic ballot.
    ///
    /// Accounts:
    /// 0. `[signer, writable]` admin (payer)
    /// 1. `[]` config PDA
    /// 2. `[writable]` binding PDA (seeds: ["proposal-binding", realm, proposal])
    /// 3. `[]` system_program
    BindProposal(BindProposalArgs),

    /// Toggle council override on a proposal binding.
    ///
    /// Accounts:
    /// 0. `[signer]` authority (admin or council_override_authority)
    /// 1. `[]` config PDA
    /// 2. `[]` proposal
    /// 3. `[writable]` binding PDA
    SetCouncilOverride { active: bool, reason_code: u16 },

    /// Create a voter weight record for a voter on a binding.
    ///
    /// Accounts:
    /// 0. `[signer, writable]` payer
    /// 1. `[]` config PDA
    /// 2. `[]` proposal
    /// 3. `[]` binding PDA
    /// 4. `[]` voter
    /// 5. `[writable]` voter_weight_record PDA (seeds: ["voter-weight", binding, voter])
    /// 6. `[]` system_program
    CreateVoterWeightRecord,

    /// Refresh a voter weight record with new reputation/allocation data.
    ///
    /// Accounts:
    /// 0. `[signer]` caller
    /// 1. `[]` config PDA
    /// 2. `[]` proposal
    /// 3. `[writable]` binding PDA
    /// 4. `[]` voter
    /// 5. `[writable]` voter_weight_record PDA
    RefreshVoterWeightRecord(RefreshVoterWeightArgs),
}

#[derive(Debug, Clone, Copy, BorshSerialize, BorshDeserialize)]
pub struct InitializeAdapterArgs {
    pub realm: Pubkey,
    pub governance_program_id: Pubkey,
    pub quadratic_voting_program: Pubkey,
    pub reputation_engine_program: Pubkey,
    pub council_override_authority: Option<Pubkey>,
    pub min_reputation_bps: u16,
    pub max_reputation_bps: u16,
}

#[derive(Debug, Clone, Copy, BorshSerialize, BorshDeserialize)]
pub struct BindProposalArgs {
    pub proposal: Pubkey,
    pub quadratic_ballot: Pubkey,
    pub governing_token_mint: Pubkey,
    pub council_override_enabled: bool,
    pub default_weight_expiry_slot: u64,
}

#[derive(Debug, Clone, Copy, BorshSerialize, BorshDeserialize)]
pub struct RefreshVoterWeightArgs {
    pub token_amount_allocated: u64,
    pub qv_votes_allocated: u32,
    pub reputation_multiplier_bps: u16,
    pub voter_weight_expiry: Option<u64>,
    pub weight_action: Option<VoterWeightAction>,
    pub weight_action_target: Option<Pubkey>,
}
