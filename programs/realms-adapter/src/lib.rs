//! Realms adapter program: binds Realms proposals to quadratic ballots and publishes voter weight records.
//!
//! Layout is modular: state, events, errors, and math live in separate modules.

use anchor_lang::prelude::*;

mod errors;
mod events;
mod math;
mod state;

pub use errors::*;
pub use events::*;
pub use state::*;

use math::{compute_effective_weight, integer_sqrt_u64};

declare_id!("11111111111111111111111111111111");

#[program]
pub mod realms_adapter {
    use super::*;

    pub fn initialize_adapter(
        ctx: Context<InitializeAdapter>,
        args: InitializeAdapterArgs,
    ) -> Result<()> {
        require!(
            args.min_reputation_bps > 0
                && args.min_reputation_bps <= args.max_reputation_bps
                && args.max_reputation_bps <= 20_000,
            RealmsAdapterError::InvalidReputationBounds
        );

        let config = &mut ctx.accounts.config;
        config.realm = args.realm;
        config.admin = ctx.accounts.admin.key();
        config.governance_program_id = args.governance_program_id;
        config.quadratic_voting_program = args.quadratic_voting_program;
        config.reputation_engine_program = args.reputation_engine_program;
        config.council_override_authority = args
            .council_override_authority
            .unwrap_or(ctx.accounts.admin.key());
        config.bump = ctx.bumps.config;
        config.min_reputation_bps = args.min_reputation_bps;
        config.max_reputation_bps = args.max_reputation_bps;

        emit!(AdapterInitializedEvent {
            realm: config.realm,
            admin: config.admin,
        });

        Ok(())
    }

    pub fn bind_proposal(ctx: Context<BindProposal>, args: BindProposalArgs) -> Result<()> {
        let binding = &mut ctx.accounts.binding;
        let config = &ctx.accounts.config;

        binding.adapter_config = config.key();
        binding.realm = config.realm;
        binding.proposal = args.proposal;
        binding.quadratic_ballot = args.quadratic_ballot;
        binding.governing_token_mint = args.governing_token_mint;
        binding.bump = ctx.bumps.binding;
        binding.council_override_enabled = args.council_override_enabled;
        binding.council_override_active = false;
        binding.council_override_reason_code = 0;
        binding.default_weight_expiry_slot = args.default_weight_expiry_slot;
        binding.last_weight_refresh_slot = 0;

        emit!(ProposalBoundEvent {
            realm: binding.realm,
            proposal: binding.proposal,
            quadratic_ballot: binding.quadratic_ballot,
        });

        Ok(())
    }

    pub fn set_council_override(
        ctx: Context<SetCouncilOverride>,
        active: bool,
        reason_code: u16,
    ) -> Result<()> {
        let signer = ctx.accounts.authority.key();
        let config = &ctx.accounts.config;
        require!(
            signer == config.admin || signer == config.council_override_authority,
            RealmsAdapterError::Unauthorized
        );

        let binding = &mut ctx.accounts.binding;
        require!(
            binding.council_override_enabled,
            RealmsAdapterError::CouncilOverrideDisabled
        );
        binding.council_override_active = active;
        binding.council_override_reason_code = reason_code;

        emit!(CouncilOverrideUpdatedEvent {
            proposal: binding.proposal,
            active,
            reason_code,
        });

        Ok(())
    }

    pub fn create_voter_weight_record(ctx: Context<CreateVoterWeightRecord>) -> Result<()> {
        let record = &mut ctx.accounts.voter_weight_record;
        let binding = &ctx.accounts.binding;

        record.binding = binding.key();
        record.voter = ctx.accounts.voter.key();
        record.governing_token_mint = binding.governing_token_mint;
        record.bump = ctx.bumps.voter_weight_record;
        record.voter_weight = 0;
        record.voter_weight_expiry_slot = None;
        record.weight_action_target = None;
        record.token_amount_allocated = 0;
        record.qv_votes_allocated = 0;
        record.reputation_multiplier_bps = 0;
        record.last_updated_slot = Clock::get()?.slot;
        record.council_override_active = binding.council_override_active;

        Ok(())
    }

    pub fn refresh_voter_weight_record(
        ctx: Context<RefreshVoterWeightRecord>,
        args: RefreshVoterWeightArgs,
    ) -> Result<()> {
        let config = &ctx.accounts.config;
        require!(
            args.reputation_multiplier_bps >= config.min_reputation_bps
                && args.reputation_multiplier_bps <= config.max_reputation_bps,
            RealmsAdapterError::InvalidReputationBounds
        );

        let binding = &mut ctx.accounts.binding;
        let record = &mut ctx.accounts.voter_weight_record;
        require!(
            record.governing_token_mint == binding.governing_token_mint,
            RealmsAdapterError::MintMismatch
        );

        let qv_component = if args.qv_votes_allocated > 0 {
            args.qv_votes_allocated as u64
        } else {
            integer_sqrt_u64(args.token_amount_allocated)
        };

        let effective_weight = compute_effective_weight(qv_component, args.reputation_multiplier_bps)?;
        let expiry_slot = args
            .voter_weight_expiry_slot
            .or(Some(binding.default_weight_expiry_slot))
            .filter(|slot| *slot > 0);

        record.voter_weight = effective_weight;
        record.voter_weight_expiry_slot = expiry_slot;
        record.weight_action_target = args.weight_action_target;
        record.token_amount_allocated = args.token_amount_allocated;
        record.qv_votes_allocated = args.qv_votes_allocated;
        record.reputation_multiplier_bps = args.reputation_multiplier_bps;
        record.last_updated_slot = Clock::get()?.slot;
        record.council_override_active = binding.council_override_active;

        binding.last_weight_refresh_slot = record.last_updated_slot;

        emit!(VoterWeightRecordRefreshedEvent {
            proposal: binding.proposal,
            voter: record.voter,
            voter_weight: record.voter_weight,
            qv_component,
            reputation_multiplier_bps: record.reputation_multiplier_bps,
            council_override_active: record.council_override_active,
        });

        Ok(())
    }
}

// --- Instruction args (kept in lib for dispatch) ---

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct InitializeAdapterArgs {
    pub realm: Pubkey,
    pub governance_program_id: Pubkey,
    pub quadratic_voting_program: Pubkey,
    pub reputation_engine_program: Pubkey,
    pub council_override_authority: Option<Pubkey>,
    pub min_reputation_bps: u16,
    pub max_reputation_bps: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct BindProposalArgs {
    pub proposal: Pubkey,
    pub quadratic_ballot: Pubkey,
    pub governing_token_mint: Pubkey,
    pub council_override_enabled: bool,
    pub default_weight_expiry_slot: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct RefreshVoterWeightArgs {
    pub token_amount_allocated: u64,
    pub qv_votes_allocated: u32,
    pub reputation_multiplier_bps: u16,
    pub voter_weight_expiry_slot: Option<u64>,
    pub weight_action_target: Option<Pubkey>,
}

// --- Context structs ---

#[derive(Accounts)]
#[instruction(args: InitializeAdapterArgs)]
pub struct InitializeAdapter<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + AdapterConfig::LEN,
        seeds = [b"adapter-config", args.realm.as_ref()],
        bump
    )]
    pub config: Account<'info, AdapterConfig>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(args: BindProposalArgs)]
pub struct BindProposal<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        has_one = admin @ RealmsAdapterError::Unauthorized,
    )]
    pub config: Account<'info, AdapterConfig>,
    #[account(
        init,
        payer = admin,
        space = 8 + ProposalBinding::LEN,
        seeds = [b"proposal-binding", config.realm.as_ref(), args.proposal.as_ref()],
        bump
    )]
    pub binding: Account<'info, ProposalBinding>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetCouncilOverride<'info> {
    pub authority: Signer<'info>,
    pub config: Account<'info, AdapterConfig>,
    /// CHECK: proposal pubkey only; no data is read.
    pub proposal: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"proposal-binding", config.realm.as_ref(), proposal.key().as_ref()],
        bump = binding.bump,
        constraint = binding.adapter_config == config.key() @ RealmsAdapterError::BindingConfigMismatch,
        constraint = binding.proposal == proposal.key() @ RealmsAdapterError::BindingProposalMismatch,
    )]
    pub binding: Account<'info, ProposalBinding>,
}

#[derive(Accounts)]
pub struct CreateVoterWeightRecord<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub config: Account<'info, AdapterConfig>,
    /// CHECK: proposal pubkey only; no data is read.
    pub proposal: UncheckedAccount<'info>,
    #[account(
        seeds = [b"proposal-binding", config.realm.as_ref(), proposal.key().as_ref()],
        bump = binding.bump,
        constraint = binding.adapter_config == config.key() @ RealmsAdapterError::BindingConfigMismatch,
        constraint = binding.proposal == proposal.key() @ RealmsAdapterError::BindingProposalMismatch,
    )]
    pub binding: Account<'info, ProposalBinding>,
    /// CHECK: voter pubkey only; no data is read.
    pub voter: UncheckedAccount<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + PluginVoterWeightRecord::LEN,
        seeds = [b"voter-weight", binding.key().as_ref(), voter.key().as_ref()],
        bump
    )]
    pub voter_weight_record: Account<'info, PluginVoterWeightRecord>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RefreshVoterWeightRecord<'info> {
    pub caller: Signer<'info>,
    pub config: Account<'info, AdapterConfig>,
    /// CHECK: proposal pubkey only; no data is read.
    pub proposal: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"proposal-binding", config.realm.as_ref(), proposal.key().as_ref()],
        bump = binding.bump,
        constraint = binding.adapter_config == config.key() @ RealmsAdapterError::BindingConfigMismatch,
        constraint = binding.proposal == proposal.key() @ RealmsAdapterError::BindingProposalMismatch,
    )]
    pub binding: Account<'info, ProposalBinding>,
    /// CHECK: voter pubkey only; no data is read.
    pub voter: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"voter-weight", binding.key().as_ref(), voter.key().as_ref()],
        bump = voter_weight_record.bump,
        constraint = voter_weight_record.binding == binding.key() @ RealmsAdapterError::WeightRecordBindingMismatch,
        constraint = voter_weight_record.voter == voter.key() @ RealmsAdapterError::WeightRecordVoterMismatch,
    )]
    pub voter_weight_record: Account<'info, PluginVoterWeightRecord>,
}
