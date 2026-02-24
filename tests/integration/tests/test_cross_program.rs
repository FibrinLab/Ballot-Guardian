#[macro_use]
#[path = "helpers.rs"]
mod helpers;

use borsh::BorshDeserialize;
use helpers::TestEnv;
use quadratic_voting::{
    InitializeBallotArgs, QuadraticBallot, QuadraticVotingInstruction, RegisterVoterArgs,
    VoteChoice,
};
use realms_adapter::{
    BindProposalArgs, InitializeAdapterArgs, PluginVoterWeightRecord, RealmsAdapterInstruction,
    RefreshVoterWeightArgs,
};
use reputation_engine::{
    ApplyComponentDeltaArgs, InitializeRealmConfigArgs, ReputationInstruction, ReputationProfile,
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

// ═══════════════════════════════════════════════════════════════════
// Cross-program flow tests
// ═══════════════════════════════════════════════════════════════════

/// Reputation engine sets a multiplier, QV ballot uses it for weighted tally.
#[test]
fn test_reputation_to_quadratic_voting_flow() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let member = Keypair::new();
    env.svm
        .airdrop(&member.pubkey(), 2_000_000_000)
        .unwrap();

    // ── Reputation Engine: init config + create profile + apply delta ──
    let (rep_config, _) = helpers::reputation_config_pda(&env.rep_pid, &realm);
    let ix = Instruction::new_with_borsh(
        env.rep_pid,
        &ReputationInstruction::InitializeRealmConfig(InitializeRealmConfigArgs {
            realm,
            oracle_authority: None,
            min_multiplier_bps: 5_000,
            base_multiplier_bps: 10_000,
            max_multiplier_bps: 20_000,
            participation_weight: 10,
            proposal_weight: 20,
            staking_weight: 15,
            tenure_weight: 5,
            delegation_weight: 10,
            points_per_bonus_bps: 100,
            penalty_unit_bps: 50,
            max_bonus_bps: 5_000,
            max_penalty_bps: 5_000,
        }),
        vec![
            AccountMeta::new(env.payer.pubkey(), true),
            AccountMeta::new(rep_config, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    let (profile_pda, _) =
        helpers::reputation_profile_pda(&env.rep_pid, &realm, &member.pubkey());
    let ix = Instruction::new_with_borsh(
        env.rep_pid,
        &ReputationInstruction::CreateProfile,
        vec![
            AccountMeta::new(env.payer.pubkey(), true),
            AccountMeta::new_readonly(rep_config, false),
            AccountMeta::new_readonly(member.pubkey(), false),
            AccountMeta::new(profile_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    // Apply deltas to raise the multiplier
    let ix = Instruction::new_with_borsh(
        env.rep_pid,
        &ReputationInstruction::ApplyComponentDelta(ApplyComponentDeltaArgs {
            participation_delta: 500,
            proposal_delta: 300,
            staking_delta: 200,
            tenure_delta: 100,
            delegation_delta: 50,
        }),
        vec![
            AccountMeta::new_readonly(env.payer.pubkey(), true),
            AccountMeta::new_readonly(rep_config, false),
            AccountMeta::new_readonly(member.pubkey(), false),
            AccountMeta::new(profile_pda, false),
        ],
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    // Read the computed multiplier
    let profile_acct = env.svm.get_account(&profile_pda).unwrap();
    let profile = ReputationProfile::try_from_slice(&profile_acct.data).unwrap();
    let member_multiplier = profile.multiplier_bps;
    assert!(member_multiplier > 10_000, "multiplier should be above base");

    // ── Quadratic Voting: create ballot + register voter with rep multiplier ──
    set_clock(&mut env, 50);

    let (ballot_pda, _) = helpers::ballot_pda(&env.qv_pid, &realm, &proposal);
    let ix = Instruction::new_with_borsh(
        env.qv_pid,
        &QuadraticVotingInstruction::InitializeBallot(InitializeBallotArgs {
            realm,
            proposal,
            min_reputation_bps: 5_000,
            max_reputation_bps: 20_000,
            voting_starts_at: 100,
            voting_ends_at: 200,
        }),
        vec![
            AccountMeta::new(env.payer.pubkey(), true),
            AccountMeta::new(ballot_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    let (alloc_pda, _) = helpers::allocation_pda(&env.qv_pid, &ballot_pda, &member.pubkey());
    let ix = Instruction::new_with_borsh(
        env.qv_pid,
        &QuadraticVotingInstruction::RegisterVoter(RegisterVoterArgs {
            credits_budget: 500,
            reputation_multiplier_bps: member_multiplier,
        }),
        vec![
            AccountMeta::new(env.payer.pubkey(), true),
            AccountMeta::new(ballot_pda, false),
            AccountMeta::new_readonly(member.pubkey(), false),
            AccountMeta::new(alloc_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    // Cast vote with reputation-derived multiplier
    set_clock(&mut env, 150);
    let ix = Instruction::new_with_borsh(
        env.qv_pid,
        &QuadraticVotingInstruction::CastVote {
            choice: VoteChoice::Yes,
            additional_votes: 10,
        },
        vec![
            AccountMeta::new_readonly(member.pubkey(), true),
            AccountMeta::new(ballot_pda, false),
            AccountMeta::new(alloc_pda, false),
        ],
    );
    helpers::send_tx(&mut env.svm, &[ix], &member, &[&member]).unwrap();

    // Verify weighted tally uses reputation multiplier
    let ballot_acct = env.svm.get_account(&ballot_pda).unwrap();
    let ballot = QuadraticBallot::try_from_slice(&ballot_acct.data).unwrap();
    let expected_scaled = 10u128 * member_multiplier as u128;
    assert_eq!(ballot.yes_tally_scaled, expected_scaled);
}

/// Full governance flow: rep engine -> QV -> adapter voter weight record.
#[test]
fn test_full_governance_flow() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let member = Keypair::new();
    env.svm
        .airdrop(&member.pubkey(), 2_000_000_000)
        .unwrap();

    // ── 1. Reputation Engine ──
    let (rep_config, _) = helpers::reputation_config_pda(&env.rep_pid, &realm);
    let ix = Instruction::new_with_borsh(
        env.rep_pid,
        &ReputationInstruction::InitializeRealmConfig(InitializeRealmConfigArgs {
            realm,
            oracle_authority: None,
            min_multiplier_bps: 5_000,
            base_multiplier_bps: 10_000,
            max_multiplier_bps: 20_000,
            participation_weight: 10,
            proposal_weight: 20,
            staking_weight: 15,
            tenure_weight: 5,
            delegation_weight: 10,
            points_per_bonus_bps: 100,
            penalty_unit_bps: 50,
            max_bonus_bps: 5_000,
            max_penalty_bps: 5_000,
        }),
        vec![
            AccountMeta::new(env.payer.pubkey(), true),
            AccountMeta::new(rep_config, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    let (profile_pda, _) =
        helpers::reputation_profile_pda(&env.rep_pid, &realm, &member.pubkey());
    let ix = Instruction::new_with_borsh(
        env.rep_pid,
        &ReputationInstruction::CreateProfile,
        vec![
            AccountMeta::new(env.payer.pubkey(), true),
            AccountMeta::new_readonly(rep_config, false),
            AccountMeta::new_readonly(member.pubkey(), false),
            AccountMeta::new(profile_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    let ix = Instruction::new_with_borsh(
        env.rep_pid,
        &ReputationInstruction::ApplyComponentDelta(ApplyComponentDeltaArgs {
            participation_delta: 200,
            proposal_delta: 100,
            staking_delta: 50,
            tenure_delta: 25,
            delegation_delta: 10,
        }),
        vec![
            AccountMeta::new_readonly(env.payer.pubkey(), true),
            AccountMeta::new_readonly(rep_config, false),
            AccountMeta::new_readonly(member.pubkey(), false),
            AccountMeta::new(profile_pda, false),
        ],
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    let profile_acct = env.svm.get_account(&profile_pda).unwrap();
    let profile = ReputationProfile::try_from_slice(&profile_acct.data).unwrap();
    let member_multiplier = profile.multiplier_bps;

    // ── 2. Quadratic Voting ──
    set_clock(&mut env, 50);

    let (ballot_pda, _) = helpers::ballot_pda(&env.qv_pid, &realm, &proposal);
    let ix = Instruction::new_with_borsh(
        env.qv_pid,
        &QuadraticVotingInstruction::InitializeBallot(InitializeBallotArgs {
            realm,
            proposal,
            min_reputation_bps: 5_000,
            max_reputation_bps: 20_000,
            voting_starts_at: 100,
            voting_ends_at: 200,
        }),
        vec![
            AccountMeta::new(env.payer.pubkey(), true),
            AccountMeta::new(ballot_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    let (alloc_pda, _) = helpers::allocation_pda(&env.qv_pid, &ballot_pda, &member.pubkey());
    let ix = Instruction::new_with_borsh(
        env.qv_pid,
        &QuadraticVotingInstruction::RegisterVoter(RegisterVoterArgs {
            credits_budget: 500,
            reputation_multiplier_bps: member_multiplier,
        }),
        vec![
            AccountMeta::new(env.payer.pubkey(), true),
            AccountMeta::new(ballot_pda, false),
            AccountMeta::new_readonly(member.pubkey(), false),
            AccountMeta::new(alloc_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    set_clock(&mut env, 150);
    let votes_cast: u32 = 8;
    let ix = Instruction::new_with_borsh(
        env.qv_pid,
        &QuadraticVotingInstruction::CastVote {
            choice: VoteChoice::Yes,
            additional_votes: votes_cast,
        },
        vec![
            AccountMeta::new_readonly(member.pubkey(), true),
            AccountMeta::new(ballot_pda, false),
            AccountMeta::new(alloc_pda, false),
        ],
    );
    helpers::send_tx(&mut env.svm, &[ix], &member, &[&member]).unwrap();

    // Finalize ballot
    set_clock(&mut env, 300);
    let ix = Instruction::new_with_borsh(
        env.qv_pid,
        &QuadraticVotingInstruction::FinalizeBallot,
        vec![
            AccountMeta::new_readonly(env.payer.pubkey(), true),
            AccountMeta::new(ballot_pda, false),
        ],
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    // ── 3. Realms Adapter ──
    let (adapter_config, _) = helpers::adapter_config_pda(&env.adapter_pid, &realm);
    let ix = Instruction::new_with_borsh(
        env.adapter_pid,
        &RealmsAdapterInstruction::InitializeAdapter(InitializeAdapterArgs {
            realm,
            governance_program_id: Pubkey::new_unique(),
            quadratic_voting_program: env.qv_pid,
            reputation_engine_program: env.rep_pid,
            council_override_authority: None,
            min_reputation_bps: 5_000,
            max_reputation_bps: 20_000,
        }),
        vec![
            AccountMeta::new(env.payer.pubkey(), true),
            AccountMeta::new(adapter_config, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    let governing_token_mint = Pubkey::new_unique();
    let (binding, _) = helpers::proposal_binding_pda(&env.adapter_pid, &realm, &proposal);
    let ix = Instruction::new_with_borsh(
        env.adapter_pid,
        &RealmsAdapterInstruction::BindProposal(BindProposalArgs {
            proposal,
            quadratic_ballot: ballot_pda,
            governing_token_mint,
            council_override_enabled: true,
            default_weight_expiry_slot: 1000,
        }),
        vec![
            AccountMeta::new(env.payer.pubkey(), true),
            AccountMeta::new_readonly(adapter_config, false),
            AccountMeta::new(binding, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    let (vwr, _) = helpers::voter_weight_pda(&env.adapter_pid, &binding, &member.pubkey());
    let ix = Instruction::new_with_borsh(
        env.adapter_pid,
        &RealmsAdapterInstruction::CreateVoterWeightRecord,
        vec![
            AccountMeta::new(env.payer.pubkey(), true),
            AccountMeta::new_readonly(adapter_config, false),
            AccountMeta::new_readonly(proposal, false),
            AccountMeta::new_readonly(binding, false),
            AccountMeta::new_readonly(member.pubkey(), false),
            AccountMeta::new(vwr, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    // Refresh with QV votes and rep multiplier
    let ix = Instruction::new_with_borsh(
        env.adapter_pid,
        &RealmsAdapterInstruction::RefreshVoterWeightRecord(RefreshVoterWeightArgs {
            token_amount_allocated: 0,
            qv_votes_allocated: votes_cast,
            reputation_multiplier_bps: member_multiplier,
            voter_weight_expiry: None,
            weight_action: None,
            weight_action_target: None,
        }),
        vec![
            AccountMeta::new_readonly(env.payer.pubkey(), true),
            AccountMeta::new_readonly(adapter_config, false),
            AccountMeta::new_readonly(proposal, false),
            AccountMeta::new(binding, false),
            AccountMeta::new_readonly(member.pubkey(), false),
            AccountMeta::new(vwr, false),
        ],
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    // Verify voter weight record
    let record_acct = env.svm.get_account(&vwr).unwrap();
    let record = PluginVoterWeightRecord::deserialize(&mut &record_acct.data[..]).unwrap();
    // qv_component = votes_cast = 8
    // effective_weight = 8 * member_multiplier / 10000
    let expected_weight = (votes_cast as u64) * (member_multiplier as u64) / 10_000;
    assert_eq!(record.voter_weight, expected_weight);
    assert_eq!(record.qv_votes_allocated, votes_cast);
    assert_eq!(record.reputation_multiplier_bps, member_multiplier);
}
