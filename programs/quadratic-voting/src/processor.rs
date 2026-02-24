//! Instruction handlers for the quadratic-voting program.

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

use crate::error::QuadraticVotingError;
use crate::events::*;
use crate::instruction::{InitializeBallotArgs, RegisterVoterArgs};
use crate::math::*;
use crate::state::*;

pub fn process_initialize_ballot(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: InitializeBallotArgs,
) -> ProgramResult {
    if args.voting_ends_at <= args.voting_starts_at {
        return Err(QuadraticVotingError::InvalidVotingWindow.into());
    }
    if args.min_reputation_bps < MIN_MULTIPLIER_BPS
        || args.min_reputation_bps > args.max_reputation_bps
        || args.max_reputation_bps > MAX_MULTIPLIER_BPS
    {
        return Err(QuadraticVotingError::InvalidReputationBounds.into());
    }

    let accounts_iter = &mut accounts.iter();
    let authority_info = next_account_info(accounts_iter)?;
    let ballot_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;

    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (ballot_pda, bump) = Pubkey::find_program_address(
        &[b"ballot", args.realm.as_ref(), args.proposal.as_ref()],
        program_id,
    );
    if ballot_pda != *ballot_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let space = QuadraticBallot::LEN;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            authority_info.key,
            ballot_info.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[
            authority_info.clone(),
            ballot_info.clone(),
            system_program_info.clone(),
        ],
        &[&[
            b"ballot",
            args.realm.as_ref(),
            args.proposal.as_ref(),
            &[bump],
        ]],
    )?;

    let ballot = QuadraticBallot {
        tag: AccountTag::QuadraticBallot as u8,
        authority: *authority_info.key,
        realm: args.realm,
        proposal: args.proposal,
        bump,
        min_reputation_bps: args.min_reputation_bps,
        max_reputation_bps: args.max_reputation_bps,
        voting_starts_at: args.voting_starts_at,
        voting_ends_at: args.voting_ends_at,
        finalized: false,
        total_registered_voters: 0,
        total_credits_budget: 0,
        yes_tally_scaled: 0,
        no_tally_scaled: 0,
        abstain_tally_scaled: 0,
    };
    ballot.serialize(&mut &mut ballot_info.data.borrow_mut()[..])?;

    emit_event(&BallotInitializedEvent {
        ballot: *ballot_info.key,
        realm: ballot.realm,
        proposal: ballot.proposal,
        authority: ballot.authority,
        voting_starts_at: ballot.voting_starts_at,
        voting_ends_at: ballot.voting_ends_at,
    });

    Ok(())
}

pub fn process_register_voter(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: RegisterVoterArgs,
) -> ProgramResult {
    if args.credits_budget == 0 {
        return Err(QuadraticVotingError::InvalidCreditsBudget.into());
    }

    let accounts_iter = &mut accounts.iter();
    let authority_info = next_account_info(accounts_iter)?;
    let ballot_info = next_account_info(accounts_iter)?;
    let voter_info = next_account_info(accounts_iter)?;
    let allocation_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;

    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if ballot_info.owner != program_id {
        return Err(QuadraticVotingError::InvalidAccountOwner.into());
    }

    let mut ballot = QuadraticBallot::try_from_slice(&ballot_info.data.borrow())?;
    if ballot.tag != AccountTag::QuadraticBallot as u8 {
        return Err(QuadraticVotingError::InvalidAccountTag.into());
    }
    if ballot.authority != *authority_info.key {
        return Err(QuadraticVotingError::Unauthorized.into());
    }
    if ballot.finalized {
        return Err(QuadraticVotingError::BallotFinalized.into());
    }
    require_multiplier_bounds(&ballot, args.reputation_multiplier_bps)?;

    let now = Clock::get()?.unix_timestamp;
    if now >= ballot.voting_ends_at {
        return Err(QuadraticVotingError::VotingWindowClosed.into());
    }

    let (allocation_pda, bump) = Pubkey::find_program_address(
        &[
            b"allocation",
            ballot_info.key.as_ref(),
            voter_info.key.as_ref(),
        ],
        program_id,
    );
    if allocation_pda != *allocation_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let space = VoterAllocation::LEN;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            authority_info.key,
            allocation_info.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[
            authority_info.clone(),
            allocation_info.clone(),
            system_program_info.clone(),
        ],
        &[&[
            b"allocation",
            ballot_info.key.as_ref(),
            voter_info.key.as_ref(),
            &[bump],
        ]],
    )?;

    let clock = Clock::get()?;
    let allocation = VoterAllocation {
        tag: AccountTag::VoterAllocation as u8,
        ballot: *ballot_info.key,
        voter: *voter_info.key,
        bump,
        reputation_multiplier_bps: args.reputation_multiplier_bps,
        credits_budget: args.credits_budget,
        credits_spent: 0,
        yes_votes: 0,
        no_votes: 0,
        abstain_votes: 0,
        last_updated_slot: clock.slot,
    };
    allocation.serialize(&mut &mut allocation_info.data.borrow_mut()[..])?;

    ballot.total_registered_voters = ballot
        .total_registered_voters
        .checked_add(1)
        .ok_or(ProgramError::from(QuadraticVotingError::MathOverflow))?;
    ballot.total_credits_budget = ballot
        .total_credits_budget
        .checked_add(args.credits_budget)
        .ok_or(ProgramError::from(QuadraticVotingError::MathOverflow))?;
    ballot.serialize(&mut &mut ballot_info.data.borrow_mut()[..])?;

    emit_event(&VoterRegisteredEvent {
        ballot: *ballot_info.key,
        voter: allocation.voter,
        credits_budget: allocation.credits_budget,
        reputation_multiplier_bps: allocation.reputation_multiplier_bps,
    });

    Ok(())
}

pub fn process_update_voter_reputation_snapshot(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_reputation_multiplier_bps: u16,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let authority_info = next_account_info(accounts_iter)?;
    let ballot_info = next_account_info(accounts_iter)?;
    let voter_info = next_account_info(accounts_iter)?;
    let allocation_info = next_account_info(accounts_iter)?;

    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if ballot_info.owner != program_id {
        return Err(QuadraticVotingError::InvalidAccountOwner.into());
    }
    if allocation_info.owner != program_id {
        return Err(QuadraticVotingError::InvalidAccountOwner.into());
    }

    let ballot = QuadraticBallot::try_from_slice(&ballot_info.data.borrow())?;
    if ballot.tag != AccountTag::QuadraticBallot as u8 {
        return Err(QuadraticVotingError::InvalidAccountTag.into());
    }
    if ballot.authority != *authority_info.key {
        return Err(QuadraticVotingError::Unauthorized.into());
    }
    if ballot.finalized {
        return Err(QuadraticVotingError::BallotFinalized.into());
    }
    require_multiplier_bounds(&ballot, new_reputation_multiplier_bps)?;

    let now = Clock::get()?.unix_timestamp;
    if now >= ballot.voting_starts_at {
        return Err(QuadraticVotingError::VotingAlreadyStarted.into());
    }

    // Verify allocation PDA
    let (expected_pda, _bump) = Pubkey::find_program_address(
        &[
            b"allocation",
            ballot_info.key.as_ref(),
            voter_info.key.as_ref(),
        ],
        program_id,
    );
    if expected_pda != *allocation_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let mut allocation = VoterAllocation::try_from_slice(&allocation_info.data.borrow())?;
    if allocation.tag != AccountTag::VoterAllocation as u8 {
        return Err(QuadraticVotingError::InvalidAccountTag.into());
    }
    if allocation.ballot != *ballot_info.key {
        return Err(QuadraticVotingError::AllocationBallotMismatch.into());
    }
    if allocation.voter != *voter_info.key {
        return Err(QuadraticVotingError::AllocationVoterMismatch.into());
    }
    if allocation.credits_spent != 0 {
        return Err(QuadraticVotingError::CannotChangeMultiplierAfterVoting.into());
    }

    allocation.reputation_multiplier_bps = new_reputation_multiplier_bps;
    allocation.last_updated_slot = Clock::get()?.slot;
    allocation.serialize(&mut &mut allocation_info.data.borrow_mut()[..])?;

    Ok(())
}

pub fn process_cast_vote(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    choice: VoteChoice,
    additional_votes: u32,
) -> ProgramResult {
    if additional_votes == 0 {
        return Err(QuadraticVotingError::ZeroAdditionalVotes.into());
    }

    let accounts_iter = &mut accounts.iter();
    let voter_info = next_account_info(accounts_iter)?;
    let ballot_info = next_account_info(accounts_iter)?;
    let allocation_info = next_account_info(accounts_iter)?;

    if !voter_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if ballot_info.owner != program_id {
        return Err(QuadraticVotingError::InvalidAccountOwner.into());
    }
    if allocation_info.owner != program_id {
        return Err(QuadraticVotingError::InvalidAccountOwner.into());
    }

    let clock = Clock::get()?;
    let mut ballot = QuadraticBallot::try_from_slice(&ballot_info.data.borrow())?;
    if ballot.tag != AccountTag::QuadraticBallot as u8 {
        return Err(QuadraticVotingError::InvalidAccountTag.into());
    }
    if ballot.finalized {
        return Err(QuadraticVotingError::BallotFinalized.into());
    }
    if clock.unix_timestamp < ballot.voting_starts_at {
        return Err(QuadraticVotingError::VotingNotStarted.into());
    }
    if clock.unix_timestamp > ballot.voting_ends_at {
        return Err(QuadraticVotingError::VotingWindowClosed.into());
    }

    // Verify allocation PDA
    let (expected_pda, _bump) = Pubkey::find_program_address(
        &[
            b"allocation",
            ballot_info.key.as_ref(),
            voter_info.key.as_ref(),
        ],
        program_id,
    );
    if expected_pda != *allocation_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let mut allocation = VoterAllocation::try_from_slice(&allocation_info.data.borrow())?;
    if allocation.tag != AccountTag::VoterAllocation as u8 {
        return Err(QuadraticVotingError::InvalidAccountTag.into());
    }
    if allocation.ballot != *ballot_info.key {
        return Err(QuadraticVotingError::AllocationBallotMismatch.into());
    }
    if allocation.voter != *voter_info.key {
        return Err(QuadraticVotingError::AllocationVoterMismatch.into());
    }

    let previous_votes = allocation.votes_for_choice(choice);
    let new_votes = previous_votes
        .checked_add(additional_votes)
        .ok_or(ProgramError::from(QuadraticVotingError::MathOverflow))?;

    let incremental_cost = quadratic_increment_cost(previous_votes, new_votes)?;
    let new_spent = allocation
        .credits_spent
        .checked_add(incremental_cost)
        .ok_or(ProgramError::from(QuadraticVotingError::MathOverflow))?;
    if new_spent > allocation.credits_budget {
        return Err(QuadraticVotingError::CreditBudgetExceeded.into());
    }

    allocation.set_votes_for_choice(choice, new_votes);
    allocation.credits_spent = new_spent;
    allocation.last_updated_slot = clock.slot;

    let weighted_increment =
        scaled_vote_weight(additional_votes, allocation.reputation_multiplier_bps)?;
    ballot.add_tally(choice, weighted_increment)?;

    allocation.serialize(&mut &mut allocation_info.data.borrow_mut()[..])?;
    ballot.serialize(&mut &mut ballot_info.data.borrow_mut()[..])?;

    emit_event(&VoteCastEvent {
        ballot: *ballot_info.key,
        voter: allocation.voter,
        choice,
        added_votes: additional_votes,
        incremental_cost,
        credits_spent: allocation.credits_spent,
        reputation_multiplier_bps: allocation.reputation_multiplier_bps,
        weighted_increment_scaled: weighted_increment,
    });

    Ok(())
}

pub fn process_finalize_ballot(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let authority_info = next_account_info(accounts_iter)?;
    let ballot_info = next_account_info(accounts_iter)?;

    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if ballot_info.owner != program_id {
        return Err(QuadraticVotingError::InvalidAccountOwner.into());
    }

    let mut ballot = QuadraticBallot::try_from_slice(&ballot_info.data.borrow())?;
    if ballot.tag != AccountTag::QuadraticBallot as u8 {
        return Err(QuadraticVotingError::InvalidAccountTag.into());
    }
    if ballot.authority != *authority_info.key {
        return Err(QuadraticVotingError::Unauthorized.into());
    }
    if ballot.finalized {
        return Err(QuadraticVotingError::BallotFinalized.into());
    }

    let now = Clock::get()?.unix_timestamp;
    if now < ballot.voting_ends_at {
        return Err(QuadraticVotingError::VotingStillActive.into());
    }

    ballot.finalized = true;
    ballot.serialize(&mut &mut ballot_info.data.borrow_mut()[..])?;

    emit_event(&BallotFinalizedEvent {
        ballot: *ballot_info.key,
        yes_tally_scaled: ballot.yes_tally_scaled,
        no_tally_scaled: ballot.no_tally_scaled,
        abstain_tally_scaled: ballot.abstain_tally_scaled,
    });

    Ok(())
}
