#[macro_use]
#[path = "helpers.rs"]
mod helpers;

use borsh::BorshDeserialize;
use helpers::TestEnv;
use quadratic_voting::{
    InitializeBallotArgs, QuadraticBallot, QuadraticVotingInstruction, RegisterVoterArgs,
    VoteChoice, VoterAllocation,
};
use solana_sdk::{
    clock::Clock,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_program,
};

// ── Clock helper ────────────────────────────────────────────────────

fn set_clock(env: &mut TestEnv, unix_timestamp: i64) {
    let clock = Clock {
        unix_timestamp,
        ..Clock::default()
    };
    env.svm.set_sysvar(&clock);
}

// ── Instruction builders ────────────────────────────────────────────

fn ix_init_ballot(pid: &Pubkey, authority: &Pubkey, args: InitializeBallotArgs) -> Instruction {
    let (ballot, _) = helpers::ballot_pda(pid, &args.realm, &args.proposal);
    Instruction::new_with_borsh(
        *pid,
        &QuadraticVotingInstruction::InitializeBallot(args),
        vec![
            AccountMeta::new(*authority, true),
            AccountMeta::new(ballot, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    )
}

fn ix_register_voter(
    pid: &Pubkey,
    authority: &Pubkey,
    ballot: &Pubkey,
    voter: &Pubkey,
    args: RegisterVoterArgs,
) -> Instruction {
    let (allocation, _) = helpers::allocation_pda(pid, ballot, voter);
    Instruction::new_with_borsh(
        *pid,
        &QuadraticVotingInstruction::RegisterVoter(args),
        vec![
            AccountMeta::new(*authority, true),
            AccountMeta::new(*ballot, false),
            AccountMeta::new_readonly(*voter, false),
            AccountMeta::new(allocation, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    )
}

fn ix_update_rep_snapshot(
    pid: &Pubkey,
    authority: &Pubkey,
    ballot: &Pubkey,
    voter: &Pubkey,
    new_multiplier_bps: u16,
) -> Instruction {
    let (allocation, _) = helpers::allocation_pda(pid, ballot, voter);
    Instruction::new_with_borsh(
        *pid,
        &QuadraticVotingInstruction::UpdateVoterReputationSnapshot {
            new_reputation_multiplier_bps: new_multiplier_bps,
        },
        vec![
            AccountMeta::new_readonly(*authority, true),
            AccountMeta::new_readonly(*ballot, false),
            AccountMeta::new_readonly(*voter, false),
            AccountMeta::new(allocation, false),
        ],
    )
}

fn ix_cast_vote(
    pid: &Pubkey,
    voter: &Pubkey,
    ballot: &Pubkey,
    choice: VoteChoice,
    additional_votes: u32,
) -> Instruction {
    let (allocation, _) = helpers::allocation_pda(pid, ballot, voter);
    Instruction::new_with_borsh(
        *pid,
        &QuadraticVotingInstruction::CastVote {
            choice,
            additional_votes,
        },
        vec![
            AccountMeta::new_readonly(*voter, true),
            AccountMeta::new(*ballot, false),
            AccountMeta::new(allocation, false),
        ],
    )
}

fn ix_finalize(pid: &Pubkey, authority: &Pubkey, ballot: &Pubkey) -> Instruction {
    Instruction::new_with_borsh(
        *pid,
        &QuadraticVotingInstruction::FinalizeBallot,
        vec![
            AccountMeta::new_readonly(*authority, true),
            AccountMeta::new(*ballot, false),
        ],
    )
}

// ── Default arguments ───────────────────────────────────────────────

fn default_ballot_args(realm: Pubkey, proposal: Pubkey) -> InitializeBallotArgs {
    InitializeBallotArgs {
        realm,
        proposal,
        min_reputation_bps: 5_000,
        max_reputation_bps: 20_000,
        voting_starts_at: 100,
        voting_ends_at: 200,
    }
}

/// Create ballot + register voter. Returns (ballot_pda, voter_keypair).
fn setup_ballot_with_voter(
    env: &mut TestEnv,
    realm: Pubkey,
    proposal: Pubkey,
    credits: u64,
    multiplier_bps: u16,
) -> (Pubkey, Keypair) {
    let (ballot, _) = helpers::ballot_pda(&env.qv_pid, &realm, &proposal);
    let voter = Keypair::new();
    env.svm.airdrop(&voter.pubkey(), 1_000_000_000).unwrap();

    // Set clock before voting starts
    set_clock(env, 50);

    let ix1 = ix_init_ballot(
        &env.qv_pid,
        &env.payer.pubkey(),
        default_ballot_args(realm, proposal),
    );
    helpers::send_tx(&mut env.svm, &[ix1], &env.payer, &[&env.payer]).unwrap();

    let ix2 = ix_register_voter(
        &env.qv_pid,
        &env.payer.pubkey(),
        &ballot,
        &voter.pubkey(),
        RegisterVoterArgs {
            credits_budget: credits,
            reputation_multiplier_bps: multiplier_bps,
        },
    );
    helpers::send_tx(&mut env.svm, &[ix2], &env.payer, &[&env.payer]).unwrap();

    (ballot, voter)
}

// ═══════════════════════════════════════════════════════════════════
// Happy-path tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_init_ballot() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let (ballot, _) = helpers::ballot_pda(&env.qv_pid, &realm, &proposal);

    set_clock(&mut env, 50);
    let ix = ix_init_ballot(
        &env.qv_pid,
        &env.payer.pubkey(),
        default_ballot_args(realm, proposal),
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    assert!(result.is_ok(), "{:?}", result.err());

    let account = env.svm.get_account(&ballot).unwrap();
    let data = QuadraticBallot::try_from_slice(&account.data).unwrap();
    assert_eq!(data.realm, realm);
    assert_eq!(data.proposal, proposal);
    assert_eq!(data.authority, env.payer.pubkey());
    assert!(!data.finalized);
    assert_eq!(data.voting_starts_at, 100);
    assert_eq!(data.voting_ends_at, 200);
}

#[test]
fn test_register_voter() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let (ballot, voter) = setup_ballot_with_voter(&mut env, realm, proposal, 1000, 10_000);

    let (alloc, _) = helpers::allocation_pda(&env.qv_pid, &ballot, &voter.pubkey());
    let account = env.svm.get_account(&alloc).unwrap();
    let data = VoterAllocation::try_from_slice(&account.data).unwrap();
    assert_eq!(data.voter, voter.pubkey());
    assert_eq!(data.credits_budget, 1000);
    assert_eq!(data.credits_spent, 0);
    assert_eq!(data.reputation_multiplier_bps, 10_000);

    // Check ballot counters
    let ballot_acct = env.svm.get_account(&ballot).unwrap();
    let ballot_data = QuadraticBallot::try_from_slice(&ballot_acct.data).unwrap();
    assert_eq!(ballot_data.total_registered_voters, 1);
    assert_eq!(ballot_data.total_credits_budget, 1000);
}

#[test]
fn test_update_rep_snapshot() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let (ballot, voter) = setup_ballot_with_voter(&mut env, realm, proposal, 1000, 10_000);

    // Still before voting starts (clock=50, starts_at=100)
    let ix = ix_update_rep_snapshot(
        &env.qv_pid,
        &env.payer.pubkey(),
        &ballot,
        &voter.pubkey(),
        15_000,
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    assert!(result.is_ok(), "{:?}", result.err());

    let (alloc, _) = helpers::allocation_pda(&env.qv_pid, &ballot, &voter.pubkey());
    let account = env.svm.get_account(&alloc).unwrap();
    let data = VoterAllocation::try_from_slice(&account.data).unwrap();
    assert_eq!(data.reputation_multiplier_bps, 15_000);
}

#[test]
fn test_cast_vote() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let (ballot, voter) = setup_ballot_with_voter(&mut env, realm, proposal, 1000, 10_000);

    // Move clock into voting window
    set_clock(&mut env, 150);

    let ix = ix_cast_vote(&env.qv_pid, &voter.pubkey(), &ballot, VoteChoice::Yes, 5);
    let result = helpers::send_tx(&mut env.svm, &[ix], &voter, &[&voter]);
    assert!(result.is_ok(), "{:?}", result.err());

    let (alloc, _) = helpers::allocation_pda(&env.qv_pid, &ballot, &voter.pubkey());
    let account = env.svm.get_account(&alloc).unwrap();
    let data = VoterAllocation::try_from_slice(&account.data).unwrap();
    assert_eq!(data.yes_votes, 5);
    assert_eq!(data.credits_spent, 25); // 5^2 = 25

    let ballot_acct = env.svm.get_account(&ballot).unwrap();
    let ballot_data = QuadraticBallot::try_from_slice(&ballot_acct.data).unwrap();
    // scaled: 5 * 10000 = 50000
    assert_eq!(ballot_data.yes_tally_scaled, 50_000);
}

#[test]
fn test_multi_choice_voting() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let (ballot, voter) = setup_ballot_with_voter(&mut env, realm, proposal, 1000, 10_000);

    set_clock(&mut env, 150);

    // Cast Yes votes
    let ix = ix_cast_vote(&env.qv_pid, &voter.pubkey(), &ballot, VoteChoice::Yes, 3);
    helpers::send_tx(&mut env.svm, &[ix], &voter, &[&voter]).unwrap();

    // Cast No votes
    let ix = ix_cast_vote(&env.qv_pid, &voter.pubkey(), &ballot, VoteChoice::No, 4);
    helpers::send_tx(&mut env.svm, &[ix], &voter, &[&voter]).unwrap();

    // Cast Abstain votes
    let ix = ix_cast_vote(
        &env.qv_pid,
        &voter.pubkey(),
        &ballot,
        VoteChoice::Abstain,
        2,
    );
    helpers::send_tx(&mut env.svm, &[ix], &voter, &[&voter]).unwrap();

    let (alloc, _) = helpers::allocation_pda(&env.qv_pid, &ballot, &voter.pubkey());
    let account = env.svm.get_account(&alloc).unwrap();
    let data = VoterAllocation::try_from_slice(&account.data).unwrap();
    assert_eq!(data.yes_votes, 3);
    assert_eq!(data.no_votes, 4);
    assert_eq!(data.abstain_votes, 2);
    // cost = 3^2 + 4^2 + 2^2 = 9 + 16 + 4 = 29
    assert_eq!(data.credits_spent, 29);
}

#[test]
fn test_finalize_ballot() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let (ballot, voter) = setup_ballot_with_voter(&mut env, realm, proposal, 1000, 10_000);

    // Vote
    set_clock(&mut env, 150);
    let ix = ix_cast_vote(&env.qv_pid, &voter.pubkey(), &ballot, VoteChoice::Yes, 5);
    helpers::send_tx(&mut env.svm, &[ix], &voter, &[&voter]).unwrap();

    // Move past voting end
    set_clock(&mut env, 300);

    let ix = ix_finalize(&env.qv_pid, &env.payer.pubkey(), &ballot);
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    assert!(result.is_ok(), "{:?}", result.err());

    let account = env.svm.get_account(&ballot).unwrap();
    let data = QuadraticBallot::try_from_slice(&account.data).unwrap();
    assert!(data.finalized);
}

#[test]
fn test_full_lifecycle() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    set_clock(&mut env, 50);

    // 1. Init ballot
    let (ballot, _) = helpers::ballot_pda(&env.qv_pid, &realm, &proposal);
    let ix = ix_init_ballot(
        &env.qv_pid,
        &env.payer.pubkey(),
        default_ballot_args(realm, proposal),
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    // 2. Register two voters
    let voter_a = Keypair::new();
    let voter_b = Keypair::new();
    env.svm.airdrop(&voter_a.pubkey(), 1_000_000_000).unwrap();
    env.svm.airdrop(&voter_b.pubkey(), 1_000_000_000).unwrap();

    let ix = ix_register_voter(
        &env.qv_pid,
        &env.payer.pubkey(),
        &ballot,
        &voter_a.pubkey(),
        RegisterVoterArgs {
            credits_budget: 500,
            reputation_multiplier_bps: 10_000,
        },
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    let ix = ix_register_voter(
        &env.qv_pid,
        &env.payer.pubkey(),
        &ballot,
        &voter_b.pubkey(),
        RegisterVoterArgs {
            credits_budget: 500,
            reputation_multiplier_bps: 15_000,
        },
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    // 3. Voting phase
    set_clock(&mut env, 150);

    let ix = ix_cast_vote(
        &env.qv_pid,
        &voter_a.pubkey(),
        &ballot,
        VoteChoice::Yes,
        10,
    );
    helpers::send_tx(&mut env.svm, &[ix], &voter_a, &[&voter_a]).unwrap();

    let ix = ix_cast_vote(
        &env.qv_pid,
        &voter_b.pubkey(),
        &ballot,
        VoteChoice::No,
        8,
    );
    helpers::send_tx(&mut env.svm, &[ix], &voter_b, &[&voter_b]).unwrap();

    // 4. Finalize
    set_clock(&mut env, 300);
    let ix = ix_finalize(&env.qv_pid, &env.payer.pubkey(), &ballot);
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    let account = env.svm.get_account(&ballot).unwrap();
    let data = QuadraticBallot::try_from_slice(&account.data).unwrap();
    assert!(data.finalized);
    assert_eq!(data.total_registered_voters, 2);
    assert_eq!(data.total_credits_budget, 1000);
    // voter_a: 10 votes * 10000 multiplier = 100000 yes_tally
    assert_eq!(data.yes_tally_scaled, 100_000);
    // voter_b: 8 votes * 15000 multiplier = 120000 no_tally
    assert_eq!(data.no_tally_scaled, 120_000);
}

// ═══════════════════════════════════════════════════════════════════
// Error-case tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_invalid_time_window() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let mut args = default_ballot_args(realm, proposal);
    args.voting_starts_at = 200;
    args.voting_ends_at = 100; // ends before start

    let ix = ix_init_ballot(&env.qv_pid, &env.payer.pubkey(), args);
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    helpers::assert_custom_error(&result, 1); // InvalidVotingWindow
}

#[test]
fn test_invalid_rep_bounds() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    let mut args = default_ballot_args(realm, proposal);
    args.min_reputation_bps = 15_000;
    args.max_reputation_bps = 10_000; // max < min

    let ix = ix_init_ballot(&env.qv_pid, &env.payer.pubkey(), args);
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    helpers::assert_custom_error(&result, 2); // InvalidReputationBounds
}

#[test]
fn test_zero_credits() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    set_clock(&mut env, 50);
    let (ballot, _) = helpers::ballot_pda(&env.qv_pid, &realm, &proposal);
    let ix = ix_init_ballot(
        &env.qv_pid,
        &env.payer.pubkey(),
        default_ballot_args(realm, proposal),
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    let voter = Pubkey::new_unique();
    let ix = ix_register_voter(
        &env.qv_pid,
        &env.payer.pubkey(),
        &ballot,
        &voter,
        RegisterVoterArgs {
            credits_budget: 0,
            reputation_multiplier_bps: 10_000,
        },
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    helpers::assert_custom_error(&result, 3); // InvalidCreditsBudget
}

#[test]
fn test_unauthorized_register() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();

    set_clock(&mut env, 50);
    let (ballot, _) = helpers::ballot_pda(&env.qv_pid, &realm, &proposal);
    let ix = ix_init_ballot(
        &env.qv_pid,
        &env.payer.pubkey(),
        default_ballot_args(realm, proposal),
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    let impostor = Keypair::new();
    env.svm
        .airdrop(&impostor.pubkey(), 1_000_000_000)
        .unwrap();
    let voter = Pubkey::new_unique();
    let ix = ix_register_voter(
        &env.qv_pid,
        &impostor.pubkey(),
        &ballot,
        &voter,
        RegisterVoterArgs {
            credits_budget: 100,
            reputation_multiplier_bps: 10_000,
        },
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &impostor, &[&impostor]);
    helpers::assert_custom_error(&result, 0); // Unauthorized
}

#[test]
fn test_voting_window_closed() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let (ballot, voter) = setup_ballot_with_voter(&mut env, realm, proposal, 1000, 10_000);

    // Past voting end
    set_clock(&mut env, 300);

    let ix = ix_cast_vote(&env.qv_pid, &voter.pubkey(), &ballot, VoteChoice::Yes, 1);
    let result = helpers::send_tx(&mut env.svm, &[ix], &voter, &[&voter]);
    helpers::assert_custom_error(&result, 5); // VotingWindowClosed
}

#[test]
fn test_voting_not_started() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let (ballot, voter) = setup_ballot_with_voter(&mut env, realm, proposal, 1000, 10_000);

    // Clock still at 50, before voting_starts_at=100
    let ix = ix_cast_vote(&env.qv_pid, &voter.pubkey(), &ballot, VoteChoice::Yes, 1);
    let result = helpers::send_tx(&mut env.svm, &[ix], &voter, &[&voter]);
    helpers::assert_custom_error(&result, 4); // VotingNotStarted
}

#[test]
fn test_zero_votes() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let (ballot, voter) = setup_ballot_with_voter(&mut env, realm, proposal, 1000, 10_000);

    set_clock(&mut env, 150);

    let ix = ix_cast_vote(&env.qv_pid, &voter.pubkey(), &ballot, VoteChoice::Yes, 0);
    let result = helpers::send_tx(&mut env.svm, &[ix], &voter, &[&voter]);
    helpers::assert_custom_error(&result, 10); // ZeroAdditionalVotes
}

#[test]
fn test_budget_exceeded() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    // Only 10 credits — can afford at most 3 votes (3^2=9)
    let (ballot, voter) = setup_ballot_with_voter(&mut env, realm, proposal, 10, 10_000);

    set_clock(&mut env, 150);

    // 4 votes costs 4^2 = 16 > 10
    let ix = ix_cast_vote(&env.qv_pid, &voter.pubkey(), &ballot, VoteChoice::Yes, 4);
    let result = helpers::send_tx(&mut env.svm, &[ix], &voter, &[&voter]);
    helpers::assert_custom_error(&result, 9); // CreditBudgetExceeded
}

#[test]
fn test_finalized_ballot_rejects_vote() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let (ballot, voter) = setup_ballot_with_voter(&mut env, realm, proposal, 1000, 10_000);

    // Vote
    set_clock(&mut env, 150);
    let ix = ix_cast_vote(&env.qv_pid, &voter.pubkey(), &ballot, VoteChoice::Yes, 1);
    helpers::send_tx(&mut env.svm, &[ix], &voter, &[&voter]).unwrap();

    // Finalize
    set_clock(&mut env, 300);
    let ix = ix_finalize(&env.qv_pid, &env.payer.pubkey(), &ballot);
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    // Try to vote again
    set_clock(&mut env, 150); // set clock back to window
    let ix = ix_cast_vote(&env.qv_pid, &voter.pubkey(), &ballot, VoteChoice::No, 1);
    let result = helpers::send_tx(&mut env.svm, &[ix], &voter, &[&voter]);
    helpers::assert_custom_error(&result, 7); // BallotFinalized
}

#[test]
fn test_voting_still_active() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let (ballot, _voter) = setup_ballot_with_voter(&mut env, realm, proposal, 1000, 10_000);

    // Try to finalize during voting window
    set_clock(&mut env, 150);

    let ix = ix_finalize(&env.qv_pid, &env.payer.pubkey(), &ballot);
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    helpers::assert_custom_error(&result, 6); // VotingStillActive
}

#[test]
fn test_already_started_snapshot_update() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let (ballot, voter) = setup_ballot_with_voter(&mut env, realm, proposal, 1000, 10_000);

    // Move clock into voting window (voting already started)
    set_clock(&mut env, 150);

    let ix = ix_update_rep_snapshot(
        &env.qv_pid,
        &env.payer.pubkey(),
        &ballot,
        &voter.pubkey(),
        15_000,
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    helpers::assert_custom_error(&result, 12); // VotingAlreadyStarted
}

#[test]
fn test_wrong_voter() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let (ballot, _voter) = setup_ballot_with_voter(&mut env, realm, proposal, 1000, 10_000);

    set_clock(&mut env, 150);

    // Wrong voter tries to vote — PDA won't match
    let wrong_voter = Keypair::new();
    env.svm
        .airdrop(&wrong_voter.pubkey(), 1_000_000_000)
        .unwrap();
    let ix = ix_cast_vote(
        &env.qv_pid,
        &wrong_voter.pubkey(),
        &ballot,
        VoteChoice::Yes,
        1,
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &wrong_voter, &[&wrong_voter]);
    helpers::assert_tx_err(&result);
}
