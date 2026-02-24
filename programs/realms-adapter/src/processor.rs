//! Instruction handlers for the realms-adapter program.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use crate::error::RealmsAdapterError;
use crate::events::*;
use crate::instruction::{BindProposalArgs, InitializeAdapterArgs, RefreshVoterWeightArgs};
use crate::math::{compute_effective_weight, integer_sqrt_u64};
use crate::state::*;

pub fn process_initialize_adapter(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: InitializeAdapterArgs,
) -> ProgramResult {
    if args.min_reputation_bps == 0
        || args.min_reputation_bps > args.max_reputation_bps
        || args.max_reputation_bps > 20_000
    {
        return Err(RealmsAdapterError::InvalidReputationBounds.into());
    }

    let accounts_iter = &mut accounts.iter();
    let admin_info = next_account_info(accounts_iter)?;
    let config_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;

    if !admin_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (config_pda, bump) =
        Pubkey::find_program_address(&[b"adapter-config", args.realm.as_ref()], program_id);
    if config_pda != *config_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let space = AdapterConfig::LEN;
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
        &[&[b"adapter-config", args.realm.as_ref(), &[bump]]],
    )?;

    let config = AdapterConfig {
        tag: AccountTag::AdapterConfig as u8,
        realm: args.realm,
        admin: *admin_info.key,
        governance_program_id: args.governance_program_id,
        quadratic_voting_program: args.quadratic_voting_program,
        reputation_engine_program: args.reputation_engine_program,
        council_override_authority: args
            .council_override_authority
            .unwrap_or(*admin_info.key),
        bump,
        min_reputation_bps: args.min_reputation_bps,
        max_reputation_bps: args.max_reputation_bps,
    };
    config.serialize(&mut &mut config_info.data.borrow_mut()[..])?;

    emit_event(&AdapterInitializedEvent {
        realm: config.realm,
        admin: config.admin,
    });

    Ok(())
}

pub fn process_bind_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: BindProposalArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let admin_info = next_account_info(accounts_iter)?;
    let config_info = next_account_info(accounts_iter)?;
    let binding_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;

    if !admin_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if config_info.owner != program_id {
        return Err(RealmsAdapterError::InvalidAccountOwner.into());
    }

    let config = AdapterConfig::try_from_slice(&config_info.data.borrow())?;
    if config.tag != AccountTag::AdapterConfig as u8 {
        return Err(RealmsAdapterError::InvalidAccountTag.into());
    }
    if config.admin != *admin_info.key {
        return Err(RealmsAdapterError::Unauthorized.into());
    }

    let (binding_pda, bump) = Pubkey::find_program_address(
        &[
            b"proposal-binding",
            config.realm.as_ref(),
            args.proposal.as_ref(),
        ],
        program_id,
    );
    if binding_pda != *binding_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let space = ProposalBinding::LEN;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            admin_info.key,
            binding_info.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[
            admin_info.clone(),
            binding_info.clone(),
            system_program_info.clone(),
        ],
        &[&[
            b"proposal-binding",
            config.realm.as_ref(),
            args.proposal.as_ref(),
            &[bump],
        ]],
    )?;

    let binding = ProposalBinding {
        tag: AccountTag::ProposalBinding as u8,
        adapter_config: *config_info.key,
        realm: config.realm,
        proposal: args.proposal,
        quadratic_ballot: args.quadratic_ballot,
        governing_token_mint: args.governing_token_mint,
        bump,
        council_override_enabled: args.council_override_enabled,
        council_override_active: false,
        council_override_reason_code: 0,
        default_weight_expiry_slot: args.default_weight_expiry_slot,
        last_weight_refresh_slot: 0,
    };
    binding.serialize(&mut &mut binding_info.data.borrow_mut()[..])?;

    emit_event(&ProposalBoundEvent {
        realm: binding.realm,
        proposal: binding.proposal,
        quadratic_ballot: binding.quadratic_ballot,
    });

    Ok(())
}

pub fn process_set_council_override(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    active: bool,
    reason_code: u16,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let authority_info = next_account_info(accounts_iter)?;
    let config_info = next_account_info(accounts_iter)?;
    let proposal_info = next_account_info(accounts_iter)?;
    let binding_info = next_account_info(accounts_iter)?;

    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if config_info.owner != program_id {
        return Err(RealmsAdapterError::InvalidAccountOwner.into());
    }
    if binding_info.owner != program_id {
        return Err(RealmsAdapterError::InvalidAccountOwner.into());
    }

    let config = AdapterConfig::try_from_slice(&config_info.data.borrow())?;
    if config.tag != AccountTag::AdapterConfig as u8 {
        return Err(RealmsAdapterError::InvalidAccountTag.into());
    }

    let signer = *authority_info.key;
    if signer != config.admin && signer != config.council_override_authority {
        return Err(RealmsAdapterError::Unauthorized.into());
    }

    // Verify binding PDA
    let (expected_pda, _bump) = Pubkey::find_program_address(
        &[
            b"proposal-binding",
            config.realm.as_ref(),
            proposal_info.key.as_ref(),
        ],
        program_id,
    );
    if expected_pda != *binding_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let mut binding = ProposalBinding::try_from_slice(&binding_info.data.borrow())?;
    if binding.tag != AccountTag::ProposalBinding as u8 {
        return Err(RealmsAdapterError::InvalidAccountTag.into());
    }
    if binding.adapter_config != *config_info.key {
        return Err(RealmsAdapterError::BindingConfigMismatch.into());
    }
    if binding.proposal != *proposal_info.key {
        return Err(RealmsAdapterError::BindingProposalMismatch.into());
    }
    if !binding.council_override_enabled {
        return Err(RealmsAdapterError::CouncilOverrideDisabled.into());
    }

    binding.council_override_active = active;
    binding.council_override_reason_code = reason_code;
    binding.serialize(&mut &mut binding_info.data.borrow_mut()[..])?;

    emit_event(&CouncilOverrideUpdatedEvent {
        proposal: binding.proposal,
        active,
        reason_code,
    });

    Ok(())
}

pub fn process_create_voter_weight_record(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_info = next_account_info(accounts_iter)?;
    let config_info = next_account_info(accounts_iter)?;
    let proposal_info = next_account_info(accounts_iter)?;
    let binding_info = next_account_info(accounts_iter)?;
    let voter_info = next_account_info(accounts_iter)?;
    let record_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;

    if !payer_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if config_info.owner != program_id {
        return Err(RealmsAdapterError::InvalidAccountOwner.into());
    }
    if binding_info.owner != program_id {
        return Err(RealmsAdapterError::InvalidAccountOwner.into());
    }

    let config = AdapterConfig::try_from_slice(&config_info.data.borrow())?;
    if config.tag != AccountTag::AdapterConfig as u8 {
        return Err(RealmsAdapterError::InvalidAccountTag.into());
    }

    // Verify binding PDA
    let (expected_binding_pda, _) = Pubkey::find_program_address(
        &[
            b"proposal-binding",
            config.realm.as_ref(),
            proposal_info.key.as_ref(),
        ],
        program_id,
    );
    if expected_binding_pda != *binding_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let binding = ProposalBinding::try_from_slice(&binding_info.data.borrow())?;
    if binding.tag != AccountTag::ProposalBinding as u8 {
        return Err(RealmsAdapterError::InvalidAccountTag.into());
    }
    if binding.adapter_config != *config_info.key {
        return Err(RealmsAdapterError::BindingConfigMismatch.into());
    }
    if binding.proposal != *proposal_info.key {
        return Err(RealmsAdapterError::BindingProposalMismatch.into());
    }

    // Derive and verify voter weight record PDA
    let (record_pda, bump) = Pubkey::find_program_address(
        &[
            b"voter-weight",
            binding_info.key.as_ref(),
            voter_info.key.as_ref(),
        ],
        program_id,
    );
    if record_pda != *record_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let space = PluginVoterWeightRecord::LEN;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            payer_info.key,
            record_info.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[
            payer_info.clone(),
            record_info.clone(),
            system_program_info.clone(),
        ],
        &[&[
            b"voter-weight",
            binding_info.key.as_ref(),
            voter_info.key.as_ref(),
            &[bump],
        ]],
    )?;

    let clock = Clock::get()?;
    let record = PluginVoterWeightRecord {
        tag: AccountTag::PluginVoterWeightRecord as u8,
        binding: *binding_info.key,
        voter: *voter_info.key,
        governing_token_mint: binding.governing_token_mint,
        bump,
        voter_weight: 0,
        voter_weight_expiry_slot: None,
        weight_action_target: None,
        token_amount_allocated: 0,
        qv_votes_allocated: 0,
        reputation_multiplier_bps: 0,
        last_updated_slot: clock.slot,
        council_override_active: binding.council_override_active,
    };
    record.serialize(&mut &mut record_info.data.borrow_mut()[..])?;

    Ok(())
}

pub fn process_refresh_voter_weight_record(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: RefreshVoterWeightArgs,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let caller_info = next_account_info(accounts_iter)?;
    let config_info = next_account_info(accounts_iter)?;
    let proposal_info = next_account_info(accounts_iter)?;
    let binding_info = next_account_info(accounts_iter)?;
    let voter_info = next_account_info(accounts_iter)?;
    let record_info = next_account_info(accounts_iter)?;

    if !caller_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if config_info.owner != program_id {
        return Err(RealmsAdapterError::InvalidAccountOwner.into());
    }
    if binding_info.owner != program_id {
        return Err(RealmsAdapterError::InvalidAccountOwner.into());
    }
    if record_info.owner != program_id {
        return Err(RealmsAdapterError::InvalidAccountOwner.into());
    }

    let config = AdapterConfig::try_from_slice(&config_info.data.borrow())?;
    if config.tag != AccountTag::AdapterConfig as u8 {
        return Err(RealmsAdapterError::InvalidAccountTag.into());
    }
    if args.reputation_multiplier_bps < config.min_reputation_bps
        || args.reputation_multiplier_bps > config.max_reputation_bps
    {
        return Err(RealmsAdapterError::InvalidReputationBounds.into());
    }

    // Verify binding PDA
    let (expected_binding_pda, _) = Pubkey::find_program_address(
        &[
            b"proposal-binding",
            config.realm.as_ref(),
            proposal_info.key.as_ref(),
        ],
        program_id,
    );
    if expected_binding_pda != *binding_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let mut binding = ProposalBinding::try_from_slice(&binding_info.data.borrow())?;
    if binding.tag != AccountTag::ProposalBinding as u8 {
        return Err(RealmsAdapterError::InvalidAccountTag.into());
    }
    if binding.adapter_config != *config_info.key {
        return Err(RealmsAdapterError::BindingConfigMismatch.into());
    }
    if binding.proposal != *proposal_info.key {
        return Err(RealmsAdapterError::BindingProposalMismatch.into());
    }

    // Verify voter weight record PDA
    let (expected_record_pda, _) = Pubkey::find_program_address(
        &[
            b"voter-weight",
            binding_info.key.as_ref(),
            voter_info.key.as_ref(),
        ],
        program_id,
    );
    if expected_record_pda != *record_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let mut record = PluginVoterWeightRecord::try_from_slice(&record_info.data.borrow())?;
    if record.tag != AccountTag::PluginVoterWeightRecord as u8 {
        return Err(RealmsAdapterError::InvalidAccountTag.into());
    }
    if record.binding != *binding_info.key {
        return Err(RealmsAdapterError::WeightRecordBindingMismatch.into());
    }
    if record.voter != *voter_info.key {
        return Err(RealmsAdapterError::WeightRecordVoterMismatch.into());
    }
    if record.governing_token_mint != binding.governing_token_mint {
        return Err(RealmsAdapterError::MintMismatch.into());
    }

    let qv_component = if args.qv_votes_allocated > 0 {
        args.qv_votes_allocated as u64
    } else {
        integer_sqrt_u64(args.token_amount_allocated)
    };

    let effective_weight =
        compute_effective_weight(qv_component, args.reputation_multiplier_bps)?;
    let expiry_slot = args
        .voter_weight_expiry_slot
        .or(Some(binding.default_weight_expiry_slot))
        .filter(|slot| *slot > 0);

    let clock = Clock::get()?;
    record.voter_weight = effective_weight;
    record.voter_weight_expiry_slot = expiry_slot;
    record.weight_action_target = args.weight_action_target;
    record.token_amount_allocated = args.token_amount_allocated;
    record.qv_votes_allocated = args.qv_votes_allocated;
    record.reputation_multiplier_bps = args.reputation_multiplier_bps;
    record.last_updated_slot = clock.slot;
    record.council_override_active = binding.council_override_active;

    record.serialize(&mut &mut record_info.data.borrow_mut()[..])?;

    binding.last_weight_refresh_slot = record.last_updated_slot;
    binding.serialize(&mut &mut binding_info.data.borrow_mut()[..])?;

    emit_event(&VoterWeightRecordRefreshedEvent {
        proposal: binding.proposal,
        voter: record.voter,
        voter_weight: record.voter_weight,
        qv_component,
        reputation_multiplier_bps: record.reputation_multiplier_bps,
        council_override_active: record.council_override_active,
    });

    Ok(())
}
