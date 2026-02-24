//! Reputation engine program: behavior-based scoring and bounded multiplier computation.
//!
//! Layout is modular: state, events, errors, and helpers live in separate modules.

use anchor_lang::prelude::*;

mod errors;
mod events;
mod helpers;
mod state;

pub use errors::*;
pub use events::*;
pub use state::*;

use helpers::{apply_delta_u32, recompute_multiplier, require_authorized_updater};

declare_id!("11111111111111111111111111111111");

#[program]
pub mod reputation_engine {
    use super::*;

    pub fn initialize_realm_config(
        ctx: Context<InitializeRealmConfig>,
        args: InitializeRealmConfigArgs,
    ) -> Result<()> {
        require!(
            args.min_multiplier_bps > 0
                && args.min_multiplier_bps <= args.base_multiplier_bps
                && args.base_multiplier_bps <= args.max_multiplier_bps,
            ReputationError::InvalidMultiplierConfig
        );
        require!(
            args.max_multiplier_bps <= 20_000 && args.min_multiplier_bps >= 5_000,
            ReputationError::InvalidMultiplierConfig
        );
        require!(
            args.points_per_bonus_bps > 0 && args.penalty_unit_bps > 0,
            ReputationError::InvalidScoringConfig
        );

        let config = &mut ctx.accounts.config;
        config.realm = args.realm;
        config.admin = ctx.accounts.admin.key();
        config.oracle_authority = args.oracle_authority.unwrap_or(ctx.accounts.admin.key());
        config.bump = ctx.bumps.config;
        config.min_multiplier_bps = args.min_multiplier_bps;
        config.base_multiplier_bps = args.base_multiplier_bps;
        config.max_multiplier_bps = args.max_multiplier_bps;
        config.participation_weight = args.participation_weight;
        config.proposal_weight = args.proposal_weight;
        config.staking_weight = args.staking_weight;
        config.tenure_weight = args.tenure_weight;
        config.delegation_weight = args.delegation_weight;
        config.points_per_bonus_bps = args.points_per_bonus_bps;
        config.penalty_unit_bps = args.penalty_unit_bps;
        config.max_bonus_bps = args.max_bonus_bps;
        config.max_penalty_bps = args.max_penalty_bps;

        emit!(RealmConfigInitializedEvent {
            realm: config.realm,
            admin: config.admin,
            oracle_authority: config.oracle_authority,
        });

        Ok(())
    }

    pub fn set_oracle_authority(
        ctx: Context<SetOracleAuthority>,
        new_oracle_authority: Pubkey,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.oracle_authority = new_oracle_authority;
        Ok(())
    }

    pub fn create_profile(ctx: Context<CreateProfile>) -> Result<()> {
        let profile = &mut ctx.accounts.profile;
        let config = &ctx.accounts.config;

        profile.realm = config.realm;
        profile.member = ctx.accounts.member.key();
        profile.bump = ctx.bumps.profile;
        profile.participation_score = 0;
        profile.proposal_creation_score = 0;
        profile.staking_score = 0;
        profile.tenure_score = 0;
        profile.delegation_trust_score = 0;
        profile.penalties_score = 0;
        profile.multiplier_bps = config.base_multiplier_bps;
        profile.last_updated_slot = Clock::get()?.slot;

        emit!(ProfileCreatedEvent {
            realm: profile.realm,
            member: profile.member,
            multiplier_bps: profile.multiplier_bps,
        });

        Ok(())
    }

    pub fn apply_component_delta(
        ctx: Context<UpdateProfile>,
        args: ApplyComponentDeltaArgs,
    ) -> Result<()> {
        require_authorized_updater(&ctx.accounts.updater, &ctx.accounts.config)?;
        let profile = &mut ctx.accounts.profile;

        profile.participation_score =
            apply_delta_u32(profile.participation_score, args.participation_delta)?;
        profile.proposal_creation_score =
            apply_delta_u32(profile.proposal_creation_score, args.proposal_delta)?;
        profile.staking_score = apply_delta_u32(profile.staking_score, args.staking_delta)?;
        profile.tenure_score = apply_delta_u32(profile.tenure_score, args.tenure_delta)?;
        profile.delegation_trust_score =
            apply_delta_u32(profile.delegation_trust_score, args.delegation_delta)?;

        recompute_multiplier(&ctx.accounts.config, profile)?;
        profile.last_updated_slot = Clock::get()?.slot;

        emit!(ProfileUpdatedEvent {
            realm: profile.realm,
            member: profile.member,
            multiplier_bps: profile.multiplier_bps,
            penalties_score: profile.penalties_score,
        });

        Ok(())
    }

    pub fn apply_penalty(ctx: Context<UpdateProfile>, penalty_points: u32, reason_code: u16) -> Result<()> {
        require_authorized_updater(&ctx.accounts.updater, &ctx.accounts.config)?;
        require!(penalty_points > 0, ReputationError::InvalidPenalty);

        let profile = &mut ctx.accounts.profile;
        profile.penalties_score = profile
            .penalties_score
            .checked_add(penalty_points)
            .ok_or(ReputationError::MathOverflow)?;
        recompute_multiplier(&ctx.accounts.config, profile)?;
        profile.last_updated_slot = Clock::get()?.slot;

        emit!(PenaltyAppliedEvent {
            realm: profile.realm,
            member: profile.member,
            penalty_points,
            reason_code,
            multiplier_bps: profile.multiplier_bps,
        });

        Ok(())
    }

    pub fn recalculate_profile(ctx: Context<UpdateProfile>) -> Result<()> {
        require_authorized_updater(&ctx.accounts.updater, &ctx.accounts.config)?;
        let profile = &mut ctx.accounts.profile;
        recompute_multiplier(&ctx.accounts.config, profile)?;
        profile.last_updated_slot = Clock::get()?.slot;
        Ok(())
    }

    pub fn snapshot_multiplier(ctx: Context<ReadProfile>) -> Result<()> {
        let profile = &ctx.accounts.profile;
        emit!(MultiplierSnapshotEvent {
            realm: profile.realm,
            member: profile.member,
            multiplier_bps: profile.multiplier_bps,
            slot: Clock::get()?.slot,
        });
        Ok(())
    }
}

// --- Instruction args (kept in lib for dispatch) ---

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct InitializeRealmConfigArgs {
    pub realm: Pubkey,
    pub oracle_authority: Option<Pubkey>,
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

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct ApplyComponentDeltaArgs {
    pub participation_delta: i32,
    pub proposal_delta: i32,
    pub staking_delta: i32,
    pub tenure_delta: i32,
    pub delegation_delta: i32,
}

// --- Context structs ---

#[derive(Accounts)]
#[instruction(args: InitializeRealmConfigArgs)]
pub struct InitializeRealmConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + RealmReputationConfig::LEN,
        seeds = [b"reputation-config", args.realm.as_ref()],
        bump
    )]
    pub config: Account<'info, RealmReputationConfig>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetOracleAuthority<'info> {
    pub admin: Signer<'info>,
    #[account(
        mut,
        has_one = admin @ ReputationError::Unauthorized,
    )]
    pub config: Account<'info, RealmReputationConfig>,
}

#[derive(Accounts)]
pub struct CreateProfile<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub config: Account<'info, RealmReputationConfig>,
    /// CHECK: member pubkey only; no data is read.
    pub member: UncheckedAccount<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + ReputationProfile::LEN,
        seeds = [b"reputation-profile", config.realm.as_ref(), member.key().as_ref()],
        bump
    )]
    pub profile: Account<'info, ReputationProfile>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateProfile<'info> {
    pub updater: Signer<'info>,
    pub config: Account<'info, RealmReputationConfig>,
    /// CHECK: Member pubkey only; no data is read.
    pub member: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"reputation-profile", config.realm.as_ref(), member.key().as_ref()],
        bump = profile.bump,
        constraint = profile.realm == config.realm @ ReputationError::ProfileRealmMismatch,
        constraint = profile.member == member.key() @ ReputationError::ProfileMemberMismatch,
    )]
    pub profile: Account<'info, ReputationProfile>,
}

#[derive(Accounts)]
pub struct ReadProfile<'info> {
    pub config: Account<'info, RealmReputationConfig>,
    /// CHECK: Member pubkey only; no data is read.
    pub member: UncheckedAccount<'info>,
    #[account(
        seeds = [b"reputation-profile", config.realm.as_ref(), member.key().as_ref()],
        bump = profile.bump,
        constraint = profile.realm == config.realm @ ReputationError::ProfileRealmMismatch,
        constraint = profile.member == member.key() @ ReputationError::ProfileMemberMismatch,
    )]
    pub profile: Account<'info, ReputationProfile>,
}
