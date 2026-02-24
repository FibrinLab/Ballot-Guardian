//! Instruction handlers for the reputation-engine program.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use crate::error::ReputationError;
use crate::events::*;
use crate::helpers::{apply_delta_u32, recompute_multiplier, require_authorized_updater};
use crate::instruction::{ApplyComponentDeltaArgs, InitializeRealmConfigArgs};
use crate::state::*;

pub fn process_initialize_realm_config(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: InitializeRealmConfigArgs,
) -> ProgramResult {
    if args.min_multiplier_bps == 0
        || args.min_multiplier_bps > args.base_multiplier_bps
        || args.base_multiplier_bps > args.max_multiplier_bps
    {
        return Err(ReputationError::InvalidMultiplierConfig.into());
    }
    if args.max_multiplier_bps > 20_000 || args.min_multiplier_bps < 5_000 {
        return Err(ReputationError::InvalidMultiplierConfig.into());
    }
    if args.points_per_bonus_bps == 0 || args.penalty_unit_bps == 0 {
        return Err(ReputationError::InvalidScoringConfig.into());
    }

    let accounts_iter = &mut accounts.iter();
    let admin_info = next_account_info(accounts_iter)?;
    let config_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;

    if !admin_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (config_pda, bump) =
        Pubkey::find_program_address(&[b"reputation-config", args.realm.as_ref()], program_id);
    if config_pda != *config_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let space = RealmReputationConfig::LEN;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            admin_info.key,
            config_info.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[
            admin_info.clone(),
            config_info.clone(),
            system_program_info.clone(),
        ],
        &[&[b"reputation-config", args.realm.as_ref(), &[bump]]],
    )?;

    let config = RealmReputationConfig {
        tag: AccountTag::RealmReputationConfig as u8,
        realm: args.realm,
        admin: *admin_info.key,
        oracle_authority: args.oracle_authority.unwrap_or(*admin_info.key),
        bump,
        min_multiplier_bps: args.min_multiplier_bps,
        base_multiplier_bps: args.base_multiplier_bps,
        max_multiplier_bps: args.max_multiplier_bps,
        participation_weight: args.participation_weight,
        proposal_weight: args.proposal_weight,
        staking_weight: args.staking_weight,
        tenure_weight: args.tenure_weight,
        delegation_weight: args.delegation_weight,
        points_per_bonus_bps: args.points_per_bonus_bps,
        penalty_unit_bps: args.penalty_unit_bps,
        max_bonus_bps: args.max_bonus_bps,
        max_penalty_bps: args.max_penalty_bps,
    };
    config.serialize(&mut &mut config_info.data.borrow_mut()[..])?;

    emit_event(&RealmConfigInitializedEvent {
        realm: config.realm,
        admin: config.admin,
        oracle_authority: config.oracle_authority,
    });

    Ok(())
}

pub fn process_set_oracle_authority(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_oracle_authority: Pubkey,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let admin_info = next_account_info(accounts_iter)?;
    let config_info = next_account_info(accounts_iter)?;

    if !admin_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if config_info.owner != program_id {
        return Err(ReputationError::InvalidAccountOwner.into());
    }

    let mut config = RealmReputationConfig::try_from_slice(&config_info.data.borrow())?;
    if config.tag != AccountTag::RealmReputationConfig as u8 {
        return Err(ReputationError::InvalidAccountTag.into());
    }
    if config.admin != *admin_info.key {
        return Err(ReputationError::Unauthorized.into());
    }

    config.oracle_authority = new_oracle_authority;
    config.serialize(&mut &mut config_info.data.borrow_mut()[..])?;

    Ok(())
}

pub fn process_create_profile(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_info = next_account_info(accounts_iter)?;
    let config_info = next_account_info(accounts_iter)?;
    let member_info = next_account_info(accounts_iter)?;
    let profile_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;

    if !payer_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if config_info.owner != program_id {
        return Err(ReputationError::InvalidAccountOwner.into());
    }

    let config = RealmReputationConfig::try_from_slice(&config_info.data.borrow())?;
    if config.tag != AccountTag::RealmReputationConfig as u8 {
        return Err(ReputationError::InvalidAccountTag.into());
    }

    let (profile_pda, bump) = Pubkey::find_program_address(
        &[
            b"reputation-profile",
            config.realm.as_ref(),
            member_info.key.as_ref(),
        ],
        program_id,
    );
    if profile_pda != *profile_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let space = ReputationProfile::LEN;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            payer_info.key,
            profile_info.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[
            payer_info.clone(),
            profile_info.clone(),
            system_program_info.clone(),
        ],
        &[&[
            b"reputation-profile",
            config.realm.as_ref(),
            member_info.key.as_ref(),
            &[bump],
        ]],
    )?;

    let clock = solana_program::clock::Clock::get()?;
    let profile = ReputationProfile {
        tag: AccountTag::ReputationProfile as u8,
        realm: config.realm,
        member: *member_info.key,
        bump,
        participation_score: 0,
        proposal_creation_score: 0,
        staking_score: 0,
        tenure_score: 0,
        delegation_trust_score: 0,
        penalties_score: 0,
        multiplier_bps: config.base_multiplier_bps,
        last_updated_slot: clock.slot,
    };
    profile.serialize(&mut &mut profile_info.data.borrow_mut()[..])?;

    emit_event(&ProfileCreatedEvent {
        realm: profile.realm,
        member: profile.member,
        multiplier_bps: profile.multiplier_bps,
    });

    Ok(())
}

pub fn process_apply_component_delta(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: ApplyComponentDeltaArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let updater_info = next_account_info(accounts_iter)?;
    let config_info = next_account_info(accounts_iter)?;
    let member_info = next_account_info(accounts_iter)?;
    let profile_info = next_account_info(accounts_iter)?;

    if !updater_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if config_info.owner != program_id {
        return Err(ReputationError::InvalidAccountOwner.into());
    }
    if profile_info.owner != program_id {
        return Err(ReputationError::InvalidAccountOwner.into());
    }

    let config = RealmReputationConfig::try_from_slice(&config_info.data.borrow())?;
    if config.tag != AccountTag::RealmReputationConfig as u8 {
        return Err(ReputationError::InvalidAccountTag.into());
    }

    require_authorized_updater(updater_info.key, &config)?;

    let mut profile = ReputationProfile::try_from_slice(&profile_info.data.borrow())?;
    if profile.tag != AccountTag::ReputationProfile as u8 {
        return Err(ReputationError::InvalidAccountTag.into());
    }

    // Verify PDA
    let (expected_pda, _bump) = Pubkey::find_program_address(
        &[
            b"reputation-profile",
            config.realm.as_ref(),
            member_info.key.as_ref(),
        ],
        program_id,
    );
    if expected_pda != *profile_info.key {
        return Err(ProgramError::InvalidSeeds);
    }
    if profile.realm != config.realm {
        return Err(ReputationError::ProfileRealmMismatch.into());
    }
    if profile.member != *member_info.key {
        return Err(ReputationError::ProfileMemberMismatch.into());
    }

    profile.participation_score =
        apply_delta_u32(profile.participation_score, args.participation_delta)?;
    profile.proposal_creation_score =
        apply_delta_u32(profile.proposal_creation_score, args.proposal_delta)?;
    profile.staking_score = apply_delta_u32(profile.staking_score, args.staking_delta)?;
    profile.tenure_score = apply_delta_u32(profile.tenure_score, args.tenure_delta)?;
    profile.delegation_trust_score =
        apply_delta_u32(profile.delegation_trust_score, args.delegation_delta)?;

    recompute_multiplier(&config, &mut profile)?;
    let clock = solana_program::clock::Clock::get()?;
    profile.last_updated_slot = clock.slot;

    profile.serialize(&mut &mut profile_info.data.borrow_mut()[..])?;

    emit_event(&ProfileUpdatedEvent {
        realm: profile.realm,
        member: profile.member,
        multiplier_bps: profile.multiplier_bps,
        penalties_score: profile.penalties_score,
    });

    Ok(())
}

pub fn process_apply_penalty(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    penalty_points: u32,
    reason_code: u16,
) -> ProgramResult {
    if penalty_points == 0 {
        return Err(ReputationError::InvalidPenalty.into());
    }

    let accounts_iter = &mut accounts.iter();
    let updater_info = next_account_info(accounts_iter)?;
    let config_info = next_account_info(accounts_iter)?;
    let member_info = next_account_info(accounts_iter)?;
    let profile_info = next_account_info(accounts_iter)?;

    if !updater_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if config_info.owner != program_id {
        return Err(ReputationError::InvalidAccountOwner.into());
    }
    if profile_info.owner != program_id {
        return Err(ReputationError::InvalidAccountOwner.into());
    }

    let config = RealmReputationConfig::try_from_slice(&config_info.data.borrow())?;
    if config.tag != AccountTag::RealmReputationConfig as u8 {
        return Err(ReputationError::InvalidAccountTag.into());
    }

    require_authorized_updater(updater_info.key, &config)?;

    let mut profile = ReputationProfile::try_from_slice(&profile_info.data.borrow())?;
    if profile.tag != AccountTag::ReputationProfile as u8 {
        return Err(ReputationError::InvalidAccountTag.into());
    }

    // Verify PDA
    let (expected_pda, _bump) = Pubkey::find_program_address(
        &[
            b"reputation-profile",
            config.realm.as_ref(),
            member_info.key.as_ref(),
        ],
        program_id,
    );
    if expected_pda != *profile_info.key {
        return Err(ProgramError::InvalidSeeds);
    }
    if profile.realm != config.realm {
        return Err(ReputationError::ProfileRealmMismatch.into());
    }
    if profile.member != *member_info.key {
        return Err(ReputationError::ProfileMemberMismatch.into());
    }

    profile.penalties_score = profile
        .penalties_score
        .checked_add(penalty_points)
        .ok_or(ProgramError::from(ReputationError::MathOverflow))?;
    recompute_multiplier(&config, &mut profile)?;
    let clock = solana_program::clock::Clock::get()?;
    profile.last_updated_slot = clock.slot;

    profile.serialize(&mut &mut profile_info.data.borrow_mut()[..])?;

    emit_event(&PenaltyAppliedEvent {
        realm: profile.realm,
        member: profile.member,
        penalty_points,
        reason_code,
        multiplier_bps: profile.multiplier_bps,
    });

    Ok(())
}

pub fn process_recalculate_profile(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let updater_info = next_account_info(accounts_iter)?;
    let config_info = next_account_info(accounts_iter)?;
    let member_info = next_account_info(accounts_iter)?;
    let profile_info = next_account_info(accounts_iter)?;

    if !updater_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if config_info.owner != program_id {
        return Err(ReputationError::InvalidAccountOwner.into());
    }
    if profile_info.owner != program_id {
        return Err(ReputationError::InvalidAccountOwner.into());
    }

    let config = RealmReputationConfig::try_from_slice(&config_info.data.borrow())?;
    if config.tag != AccountTag::RealmReputationConfig as u8 {
        return Err(ReputationError::InvalidAccountTag.into());
    }

    require_authorized_updater(updater_info.key, &config)?;

    let mut profile = ReputationProfile::try_from_slice(&profile_info.data.borrow())?;
    if profile.tag != AccountTag::ReputationProfile as u8 {
        return Err(ReputationError::InvalidAccountTag.into());
    }

    // Verify PDA
    let (expected_pda, _bump) = Pubkey::find_program_address(
        &[
            b"reputation-profile",
            config.realm.as_ref(),
            member_info.key.as_ref(),
        ],
        program_id,
    );
    if expected_pda != *profile_info.key {
        return Err(ProgramError::InvalidSeeds);
    }
    if profile.realm != config.realm {
        return Err(ReputationError::ProfileRealmMismatch.into());
    }
    if profile.member != *member_info.key {
        return Err(ReputationError::ProfileMemberMismatch.into());
    }

    recompute_multiplier(&config, &mut profile)?;
    let clock = solana_program::clock::Clock::get()?;
    profile.last_updated_slot = clock.slot;

    profile.serialize(&mut &mut profile_info.data.borrow_mut()[..])?;

    Ok(())
}

pub fn process_snapshot_multiplier(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let config_info = next_account_info(accounts_iter)?;
    let member_info = next_account_info(accounts_iter)?;
    let profile_info = next_account_info(accounts_iter)?;

    if config_info.owner != program_id {
        return Err(ReputationError::InvalidAccountOwner.into());
    }
    if profile_info.owner != program_id {
        return Err(ReputationError::InvalidAccountOwner.into());
    }

    let config = RealmReputationConfig::try_from_slice(&config_info.data.borrow())?;
    if config.tag != AccountTag::RealmReputationConfig as u8 {
        return Err(ReputationError::InvalidAccountTag.into());
    }

    let profile = ReputationProfile::try_from_slice(&profile_info.data.borrow())?;
    if profile.tag != AccountTag::ReputationProfile as u8 {
        return Err(ReputationError::InvalidAccountTag.into());
    }

    // Verify PDA
    let (expected_pda, _bump) = Pubkey::find_program_address(
        &[
            b"reputation-profile",
            config.realm.as_ref(),
            member_info.key.as_ref(),
        ],
        program_id,
    );
    if expected_pda != *profile_info.key {
        return Err(ProgramError::InvalidSeeds);
    }
    if profile.realm != config.realm {
        return Err(ReputationError::ProfileRealmMismatch.into());
    }
    if profile.member != *member_info.key {
        return Err(ReputationError::ProfileMemberMismatch.into());
    }

    let clock = solana_program::clock::Clock::get()?;
    emit_event(&MultiplierSnapshotEvent {
        realm: profile.realm,
        member: profile.member,
        multiplier_bps: profile.multiplier_bps,
        slot: clock.slot,
    });

    Ok(())
}
