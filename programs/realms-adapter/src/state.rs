//! Account state for the realms-adapter program.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

/// SHA256("account:VoterWeightRecord")[..8]
pub const VOTER_WEIGHT_RECORD_DISCRIMINATOR: [u8; 8] = [46, 249, 155, 75, 153, 248, 116, 9];

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
#[borsh(use_discriminant = true)]
#[repr(u8)]
pub enum AccountTag {
    Uninitialized = 0,
    AdapterConfig = 1,
    ProposalBinding = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
#[borsh(use_discriminant = true)]
#[repr(u8)]
pub enum VoterWeightAction {
    CastVote = 0,
    CommentProposal = 1,
    CreateGovernance = 2,
    CreateProposal = 3,
    SignOffProposal = 4,
}

#[derive(Debug, Clone, Copy, BorshSerialize, BorshDeserialize)]
pub struct AdapterConfig {
    pub tag: u8,
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
    pub const LEN: usize = 1 + (32 * 6) + 1 + 2 + 2;
}

#[derive(Debug, Clone, Copy, BorshSerialize, BorshDeserialize)]
pub struct ProposalBinding {
    pub tag: u8,
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
    pub const LEN: usize = 1 + (32 * 5) + 1 + 1 + 1 + 2 + 8 + 8;
}

/// SPL-compatible voter weight record.
///
/// The first 164 bytes match the SPL Governance VoterWeightRecord layout
/// (8-byte discriminator prefix + SPL fields). Plugin extension fields follow.
#[derive(Debug, Clone, Copy)]
pub struct PluginVoterWeightRecord {
    // ── SPL-compatible portion ──
    // account_discriminator: [u8; 8] — written/read by manual ser/de
    pub realm: Pubkey,
    pub governing_token_mint: Pubkey,
    pub governing_token_owner: Pubkey,
    pub voter_weight: u64,
    pub voter_weight_expiry: Option<u64>,
    pub weight_action: Option<VoterWeightAction>,
    pub weight_action_target: Option<Pubkey>,
    pub reserved: [u8; 8],
    // ── Plugin extension fields ──
    pub binding: Pubkey,
    pub bump: u8,
    pub token_amount_allocated: u64,
    pub qv_votes_allocated: u32,
    pub reputation_multiplier_bps: u16,
    pub last_updated_slot: u64,
    pub council_override_active: bool,
}

impl PluginVoterWeightRecord {
    /// SPL portion: 8 + 32 + 32 + 32 + 8 + 9 + 2 + 33 + 8 = 164
    /// Extension:   32 + 1 + 8 + 4 + 2 + 8 + 1 = 56
    /// Total: 220
    pub const LEN: usize = 164 + 56;
}

impl BorshSerialize for PluginVoterWeightRecord {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        // Write the 8-byte discriminator prefix
        writer.write_all(&VOTER_WEIGHT_RECORD_DISCRIMINATOR)?;
        // SPL fields
        BorshSerialize::serialize(&self.realm, writer)?;
        BorshSerialize::serialize(&self.governing_token_mint, writer)?;
        BorshSerialize::serialize(&self.governing_token_owner, writer)?;
        BorshSerialize::serialize(&self.voter_weight, writer)?;
        BorshSerialize::serialize(&self.voter_weight_expiry, writer)?;
        BorshSerialize::serialize(&self.weight_action, writer)?;
        BorshSerialize::serialize(&self.weight_action_target, writer)?;
        BorshSerialize::serialize(&self.reserved, writer)?;
        // Extension fields
        BorshSerialize::serialize(&self.binding, writer)?;
        BorshSerialize::serialize(&self.bump, writer)?;
        BorshSerialize::serialize(&self.token_amount_allocated, writer)?;
        BorshSerialize::serialize(&self.qv_votes_allocated, writer)?;
        BorshSerialize::serialize(&self.reputation_multiplier_bps, writer)?;
        BorshSerialize::serialize(&self.last_updated_slot, writer)?;
        BorshSerialize::serialize(&self.council_override_active, writer)?;
        Ok(())
    }
}

impl BorshDeserialize for PluginVoterWeightRecord {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut discriminator = [0u8; 8];
        reader.read_exact(&mut discriminator)?;
        if discriminator != VOTER_WEIGHT_RECORD_DISCRIMINATOR {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid VoterWeightRecord discriminator",
            ));
        }
        Ok(Self {
            realm: BorshDeserialize::deserialize_reader(reader)?,
            governing_token_mint: BorshDeserialize::deserialize_reader(reader)?,
            governing_token_owner: BorshDeserialize::deserialize_reader(reader)?,
            voter_weight: BorshDeserialize::deserialize_reader(reader)?,
            voter_weight_expiry: BorshDeserialize::deserialize_reader(reader)?,
            weight_action: BorshDeserialize::deserialize_reader(reader)?,
            weight_action_target: BorshDeserialize::deserialize_reader(reader)?,
            reserved: BorshDeserialize::deserialize_reader(reader)?,
            binding: BorshDeserialize::deserialize_reader(reader)?,
            bump: BorshDeserialize::deserialize_reader(reader)?,
            token_amount_allocated: BorshDeserialize::deserialize_reader(reader)?,
            qv_votes_allocated: BorshDeserialize::deserialize_reader(reader)?,
            reputation_multiplier_bps: BorshDeserialize::deserialize_reader(reader)?,
            last_updated_slot: BorshDeserialize::deserialize_reader(reader)?,
            council_override_active: BorshDeserialize::deserialize_reader(reader)?,
        })
    }
}
