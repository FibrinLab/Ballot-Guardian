//! Quadratic voting program: proposal-level ballots, credit budgets, and reputation-scaled tallies.
//!
//! Layout is modular: state, events, errors, and math live in separate modules.

use anchor_lang::prelude::*;

mod errors;
mod events;
mod math;
mod state;

pub use errors::*;
pub use events::*;
pub use math::*;
pub use state::*;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod quadratic_voting {
    use super::*;

    pub fn initialize_ballot(
        ctx: Context<InitializeBallot>,
        args: InitializeBallotArgs,
    ) -> Result<()> {
        require!(
            args.voting_ends_at > args.voting_starts_at,
            QuadraticVotingError::InvalidVotingWindow
        );
        require!(
            args.min_reputation_bps >= MIN_MULTIPLIER_BPS
                && args.min_reputation_bps <= args.max_reputation_bps
                && args.max_reputation_bps <= MAX_MULTIPLIER_BPS,
            QuadraticVotingError::InvalidReputationBounds
        );

        let ballot = &mut ctx.accounts.ballot;
        ballot.authority = ctx.accounts.authority.key();
        ballot.realm = args.realm;
        ballot.proposal = args.proposal;
        ballot.bump = ctx.bumps.ballot;
        ballot.min_reputation_bps = args.min_reputation_bps;
        ballot.max_reputation_bps = args.max_reputation_bps;
        ballot.voting_starts_at = args.voting_starts_at;
        ballot.voting_ends_at = args.voting_ends_at;
        ballot.finalized = false;
        ballot.total_registered_voters = 0;
        ballot.total_credits_budget = 0;
        ballot.yes_tally_scaled = 0;
        ballot.no_tally_scaled = 0;
        ballot.abstain_tally_scaled = 0;

        emit!(BallotInitializedEvent {
            ballot: ballot.key(),
            realm: ballot.realm,
            proposal: ballot.proposal,
            authority: ballot.authority,
            voting_starts_at: ballot.voting_starts_at,
            voting_ends_at: ballot.voting_ends_at,
        });

        Ok(())
    }

    pub fn register_voter(ctx: Context<RegisterVoter>, args: RegisterVoterArgs) -> Result<()> {
        require!(args.credits_budget > 0, QuadraticVotingError::InvalidCreditsBudget);

        let ballot = &mut ctx.accounts.ballot;
        require!(!ballot.finalized, QuadraticVotingError::BallotFinalized);
        require_multiplier_bounds(ballot, args.reputation_multiplier_bps)?;

        let now = Clock::get()?.unix_timestamp;
        require!(
            now < ballot.voting_ends_at,
            QuadraticVotingError::VotingWindowClosed
        );

        let allocation = &mut ctx.accounts.allocation;
        allocation.ballot = ballot.key();
        allocation.voter = ctx.accounts.voter.key();
        allocation.bump = ctx.bumps.allocation;
        allocation.reputation_multiplier_bps = args.reputation_multiplier_bps;
        allocation.credits_budget = args.credits_budget;
        allocation.credits_spent = 0;
        allocation.yes_votes = 0;
        allocation.no_votes = 0;
        allocation.abstain_votes = 0;
        allocation.last_updated_slot = Clock::get()?.slot;

        ballot.total_registered_voters = ballot
            .total_registered_voters
            .checked_add(1)
            .ok_or(QuadraticVotingError::MathOverflow)?;
        ballot.total_credits_budget = ballot
            .total_credits_budget
            .checked_add(args.credits_budget)
            .ok_or(QuadraticVotingError::MathOverflow)?;

        emit!(VoterRegisteredEvent {
            ballot: ballot.key(),
            voter: allocation.voter,
            credits_budget: allocation.credits_budget,
            reputation_multiplier_bps: allocation.reputation_multiplier_bps,
        });

        Ok(())
    }

    pub fn update_voter_reputation_snapshot(
        ctx: Context<UpdateVoterReputationSnapshot>,
        new_reputation_multiplier_bps: u16,
    ) -> Result<()> {
        let ballot = &ctx.accounts.ballot;
        require!(!ballot.finalized, QuadraticVotingError::BallotFinalized);
        require_multiplier_bounds(ballot, new_reputation_multiplier_bps)?;

        let now = Clock::get()?.unix_timestamp;
        require!(
            now < ballot.voting_starts_at,
            QuadraticVotingError::VotingAlreadyStarted
        );

        let allocation = &mut ctx.accounts.allocation;
        require!(
            allocation.credits_spent == 0,
            QuadraticVotingError::CannotChangeMultiplierAfterVoting
        );
        allocation.reputation_multiplier_bps = new_reputation_multiplier_bps;
        allocation.last_updated_slot = Clock::get()?.slot;

        Ok(())
    }

    pub fn cast_vote(
        ctx: Context<CastVote>,
        choice: VoteChoice,
        additional_votes: u32,
    ) -> Result<()> {
        require!(additional_votes > 0, QuadraticVotingError::ZeroAdditionalVotes);

        let clock = Clock::get()?;
        let ballot = &mut ctx.accounts.ballot;
        require!(!ballot.finalized, QuadraticVotingError::BallotFinalized);
        require!(
            clock.unix_timestamp >= ballot.voting_starts_at,
            QuadraticVotingError::VotingNotStarted
        );
        require!(
            clock.unix_timestamp <= ballot.voting_ends_at,
            QuadraticVotingError::VotingWindowClosed
        );

        let allocation = &mut ctx.accounts.allocation;
        let previous_votes = allocation.votes_for_choice(choice);
        let new_votes = previous_votes
            .checked_add(additional_votes)
            .ok_or(QuadraticVotingError::MathOverflow)?;

        let incremental_cost = quadratic_increment_cost(previous_votes, new_votes)?;
        let new_spent = allocation
            .credits_spent
            .checked_add(incremental_cost)
            .ok_or(QuadraticVotingError::MathOverflow)?;
        require!(
            new_spent <= allocation.credits_budget,
            QuadraticVotingError::CreditBudgetExceeded
        );

        allocation.set_votes_for_choice(choice, new_votes);
        allocation.credits_spent = new_spent;
        allocation.last_updated_slot = clock.slot;

        let weighted_increment =
            scaled_vote_weight(additional_votes, allocation.reputation_multiplier_bps)?;
        ballot.add_tally(choice, weighted_increment)?;

        emit!(VoteCastEvent {
            ballot: ballot.key(),
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

    pub fn finalize_ballot(ctx: Context<FinalizeBallot>) -> Result<()> {
        let ballot = &mut ctx.accounts.ballot;
        require!(!ballot.finalized, QuadraticVotingError::BallotFinalized);

        let now = Clock::get()?.unix_timestamp;
        require!(
            now >= ballot.voting_ends_at,
            QuadraticVotingError::VotingStillActive
        );

        ballot.finalized = true;

        emit!(BallotFinalizedEvent {
            ballot: ballot.key(),
            yes_tally_scaled: ballot.yes_tally_scaled,
            no_tally_scaled: ballot.no_tally_scaled,
            abstain_tally_scaled: ballot.abstain_tally_scaled,
        });

        Ok(())
    }
}

// --- Instruction args (kept in lib for dispatch) ---

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct InitializeBallotArgs {
    pub realm: Pubkey,
    pub proposal: Pubkey,
    pub min_reputation_bps: u16,
    pub max_reputation_bps: u16,
    pub voting_starts_at: i64,
    pub voting_ends_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct RegisterVoterArgs {
    pub credits_budget: u64,
    pub reputation_multiplier_bps: u16,
}

// --- Context structs ---

#[derive(Accounts)]
#[instruction(args: InitializeBallotArgs)]
pub struct InitializeBallot<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + QuadraticBallot::LEN,
        seeds = [b"ballot", args.realm.as_ref(), args.proposal.as_ref()],
        bump
    )]
    pub ballot: Account<'info, QuadraticBallot>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterVoter<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        has_one = authority @ QuadraticVotingError::Unauthorized,
    )]
    pub ballot: Account<'info, QuadraticBallot>,
    /// CHECK: Voter pubkey only; no data is read.
    pub voter: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + VoterAllocation::LEN,
        seeds = [b"allocation", ballot.key().as_ref(), voter.key().as_ref()],
        bump
    )]
    pub allocation: Account<'info, VoterAllocation>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateVoterReputationSnapshot<'info> {
    pub authority: Signer<'info>,
    #[account(
        has_one = authority @ QuadraticVotingError::Unauthorized,
    )]
    pub ballot: Account<'info, QuadraticBallot>,
    /// CHECK: Voter pubkey only; no data is read.
    pub voter: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"allocation", ballot.key().as_ref(), voter.key().as_ref()],
        bump = allocation.bump,
        has_one = ballot @ QuadraticVotingError::AllocationBallotMismatch,
        constraint = allocation.voter == voter.key() @ QuadraticVotingError::AllocationVoterMismatch,
    )]
    pub allocation: Account<'info, VoterAllocation>,
}

#[derive(Accounts)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,
    #[account(mut)]
    pub ballot: Account<'info, QuadraticBallot>,
    #[account(
        mut,
        seeds = [b"allocation", ballot.key().as_ref(), voter.key().as_ref()],
        bump = allocation.bump,
        has_one = ballot @ QuadraticVotingError::AllocationBallotMismatch,
        has_one = voter @ QuadraticVotingError::AllocationVoterMismatch,
    )]
    pub allocation: Account<'info, VoterAllocation>,
}

#[derive(Accounts)]
pub struct FinalizeBallot<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        has_one = authority @ QuadraticVotingError::Unauthorized,
    )]
    pub ballot: Account<'info, QuadraticBallot>,
}
