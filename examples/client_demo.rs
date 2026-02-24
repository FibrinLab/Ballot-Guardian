//! Client demo: invokes all three Ballot Guardian programs against a local validator.
//!
//! Prerequisites:
//!   1. Build SBF programs: `cargo build-sbf`
//!   2. Start a local validator with programs loaded:
//!      ```
//!      solana-test-validator \
//!        --bpf-program <REP_PROGRAM_ID> target/deploy/reputation_engine.so \
//!        --bpf-program <QV_PROGRAM_ID>  target/deploy/quadratic_voting.so \
//!        --bpf-program <RA_PROGRAM_ID>  target/deploy/realms_adapter.so
//!      ```
//!   3. Run: `cargo run -p ballot-guardian-examples --bin client_demo`

use borsh::BorshDeserialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_program,
    transaction::Transaction,
};

use reputation_engine::{
    ApplyComponentDeltaArgs, InitializeRealmConfigArgs, ReputationInstruction, ReputationProfile,
};

use quadratic_voting::{
    InitializeBallotArgs, QuadraticBallot, QuadraticVotingInstruction, RegisterVoterArgs,
    VoteChoice,
};

use realms_adapter::{
    BindProposalArgs, InitializeAdapterArgs, PluginVoterWeightRecord, RealmsAdapterInstruction,
    RefreshVoterWeightArgs,
};

// ── Replace these with your deployed program IDs ────────────────────
// You can generate keypairs or use the ones from `cargo build-sbf` output.
const REP_PROGRAM_ID: &str = "8JpbKjoR4c7n2HqS51WjyjJrLwvVgGGsKN4o2boohdEA";
const QV_PROGRAM_ID: &str = "346RNEQcBBff4skHhQiRCPe9cDeWpaPTsT2TpQUFYomp";
const ADAPTER_PROGRAM_ID: &str = "E5CHyQY6gsxWB4cdTCSMxS3aY3J4eCVCXEe1KVTfk4Ky";

fn main() {
    println!("=== Ballot Guardian Client Demo ===\n");

    let rpc_url = "http://localhost:8899";
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

    let payer = Keypair::new();
    println!("Payer: {}", payer.pubkey());

    // Airdrop SOL
    println!("Requesting airdrop...");
    let sig = client
        .request_airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
        .expect("Airdrop failed — is solana-test-validator running?");
    client
        .confirm_transaction(&sig)
        .expect("Airdrop confirmation failed");
    println!("Airdrop confirmed.\n");

    let rep_pid: Pubkey = REP_PROGRAM_ID.parse().unwrap();
    let qv_pid: Pubkey = QV_PROGRAM_ID.parse().unwrap();
    let adapter_pid: Pubkey = ADAPTER_PROGRAM_ID.parse().unwrap();

    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let member = Keypair::new();

    // Fund member
    let sig = client
        .request_airdrop(&member.pubkey(), 2 * LAMPORTS_PER_SOL)
        .unwrap();
    client.confirm_transaction(&sig).unwrap();

    // ═══════════════════════════════════════════════════════════════
    // 1. Reputation Engine
    // ═══════════════════════════════════════════════════════════════
    println!("--- Reputation Engine ---");

    let (rep_config, _) =
        Pubkey::find_program_address(&[b"reputation-config", realm.as_ref()], &rep_pid);

    let ix = Instruction::new_with_borsh(
        rep_pid,
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
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(rep_config, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    send(&client, &[ix], &payer, &[&payer], "InitializeRealmConfig");

    let (profile_pda, _) = Pubkey::find_program_address(
        &[
            b"reputation-profile",
            realm.as_ref(),
            member.pubkey().as_ref(),
        ],
        &rep_pid,
    );
    let ix = Instruction::new_with_borsh(
        rep_pid,
        &ReputationInstruction::CreateProfile,
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(rep_config, false),
            AccountMeta::new_readonly(member.pubkey(), false),
            AccountMeta::new(profile_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    send(&client, &[ix], &payer, &[&payer], "CreateProfile");

    let ix = Instruction::new_with_borsh(
        rep_pid,
        &ReputationInstruction::ApplyComponentDelta(ApplyComponentDeltaArgs {
            participation_delta: 200,
            proposal_delta: 100,
            staking_delta: 50,
            tenure_delta: 25,
            delegation_delta: 10,
        }),
        vec![
            AccountMeta::new_readonly(payer.pubkey(), true),
            AccountMeta::new_readonly(rep_config, false),
            AccountMeta::new_readonly(member.pubkey(), false),
            AccountMeta::new(profile_pda, false),
        ],
    );
    send(&client, &[ix], &payer, &[&payer], "ApplyComponentDelta");

    let profile_acct = client.get_account(&profile_pda).unwrap();
    let profile = ReputationProfile::try_from_slice(&profile_acct.data).unwrap();
    println!(
        "  Member multiplier: {} bps (base: 10000)\n",
        profile.multiplier_bps
    );

    // ═══════════════════════════════════════════════════════════════
    // 2. Quadratic Voting
    // ═══════════════════════════════════════════════════════════════
    println!("--- Quadratic Voting ---");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let (ballot_pda, _) =
        Pubkey::find_program_address(&[b"ballot", realm.as_ref(), proposal.as_ref()], &qv_pid);

    let ix = Instruction::new_with_borsh(
        qv_pid,
        &QuadraticVotingInstruction::InitializeBallot(InitializeBallotArgs {
            realm,
            proposal,
            min_reputation_bps: 5_000,
            max_reputation_bps: 20_000,
            voting_starts_at: now - 60,  // started 1 min ago
            voting_ends_at: now + 3600,  // ends in 1 hour
        }),
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(ballot_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    send(&client, &[ix], &payer, &[&payer], "InitializeBallot");

    let (alloc_pda, _) = Pubkey::find_program_address(
        &[
            b"allocation",
            ballot_pda.as_ref(),
            member.pubkey().as_ref(),
        ],
        &qv_pid,
    );
    let ix = Instruction::new_with_borsh(
        qv_pid,
        &QuadraticVotingInstruction::RegisterVoter(RegisterVoterArgs {
            credits_budget: 500,
            reputation_multiplier_bps: profile.multiplier_bps,
        }),
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(ballot_pda, false),
            AccountMeta::new_readonly(member.pubkey(), false),
            AccountMeta::new(alloc_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    send(&client, &[ix], &payer, &[&payer], "RegisterVoter");

    let ix = Instruction::new_with_borsh(
        qv_pid,
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
    send(&client, &[ix], &member, &[&member], "CastVote (Yes, 10)");

    let ballot_acct = client.get_account(&ballot_pda).unwrap();
    let ballot = QuadraticBallot::try_from_slice(&ballot_acct.data).unwrap();
    println!(
        "  Ballot tallies — Yes: {}, No: {}, Abstain: {}\n",
        ballot.yes_tally_scaled, ballot.no_tally_scaled, ballot.abstain_tally_scaled
    );

    // ═══════════════════════════════════════════════════════════════
    // 3. Realms Adapter
    // ═══════════════════════════════════════════════════════════════
    println!("--- Realms Adapter ---");

    let (adapter_config, _) =
        Pubkey::find_program_address(&[b"adapter-config", realm.as_ref()], &adapter_pid);
    let ix = Instruction::new_with_borsh(
        adapter_pid,
        &RealmsAdapterInstruction::InitializeAdapter(InitializeAdapterArgs {
            realm,
            governance_program_id: Pubkey::new_unique(),
            quadratic_voting_program: qv_pid,
            reputation_engine_program: rep_pid,
            council_override_authority: None,
            min_reputation_bps: 5_000,
            max_reputation_bps: 20_000,
        }),
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(adapter_config, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    send(&client, &[ix], &payer, &[&payer], "InitializeAdapter");

    let governing_token_mint = Pubkey::new_unique();
    let (binding, _) = Pubkey::find_program_address(
        &[b"proposal-binding", realm.as_ref(), proposal.as_ref()],
        &adapter_pid,
    );
    let ix = Instruction::new_with_borsh(
        adapter_pid,
        &RealmsAdapterInstruction::BindProposal(BindProposalArgs {
            proposal,
            quadratic_ballot: ballot_pda,
            governing_token_mint,
            council_override_enabled: true,
            default_weight_expiry_slot: 1_000_000,
        }),
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(adapter_config, false),
            AccountMeta::new(binding, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    send(&client, &[ix], &payer, &[&payer], "BindProposal");

    let (vwr, _) = Pubkey::find_program_address(
        &[
            b"voter-weight",
            binding.as_ref(),
            member.pubkey().as_ref(),
        ],
        &adapter_pid,
    );
    let ix = Instruction::new_with_borsh(
        adapter_pid,
        &RealmsAdapterInstruction::CreateVoterWeightRecord,
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(adapter_config, false),
            AccountMeta::new_readonly(proposal, false),
            AccountMeta::new_readonly(binding, false),
            AccountMeta::new_readonly(member.pubkey(), false),
            AccountMeta::new(vwr, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    send(
        &client,
        &[ix],
        &payer,
        &[&payer],
        "CreateVoterWeightRecord",
    );

    let ix = Instruction::new_with_borsh(
        adapter_pid,
        &RealmsAdapterInstruction::RefreshVoterWeightRecord(RefreshVoterWeightArgs {
            token_amount_allocated: 0,
            qv_votes_allocated: 10,
            reputation_multiplier_bps: profile.multiplier_bps,
            voter_weight_expiry: None,
            weight_action: None,
            weight_action_target: None,
        }),
        vec![
            AccountMeta::new_readonly(payer.pubkey(), true),
            AccountMeta::new_readonly(adapter_config, false),
            AccountMeta::new_readonly(proposal, false),
            AccountMeta::new(binding, false),
            AccountMeta::new_readonly(member.pubkey(), false),
            AccountMeta::new(vwr, false),
        ],
    );
    send(
        &client,
        &[ix],
        &payer,
        &[&payer],
        "RefreshVoterWeightRecord",
    );

    let record_acct = client.get_account(&vwr).unwrap();
    let record = PluginVoterWeightRecord::deserialize(&mut &record_acct.data[..]).unwrap();
    println!("  Voter weight: {}", record.voter_weight);
    println!(
        "  Reputation multiplier used: {} bps",
        record.reputation_multiplier_bps
    );
    println!(
        "  Council override active: {}\n",
        record.council_override_active
    );

    println!("=== Demo complete ===");
}

fn send(
    client: &RpcClient,
    ixs: &[Instruction],
    payer: &Keypair,
    signers: &[&Keypair],
    label: &str,
) {
    let blockhash = client.get_latest_blockhash().unwrap();
    let tx = Transaction::new_signed_with_payer(ixs, Some(&payer.pubkey()), signers, blockhash);
    match client.send_and_confirm_transaction(&tx) {
        Ok(sig) => println!("  {} -> {}", label, sig),
        Err(e) => {
            eprintln!("  {} FAILED: {}", label, e);
            std::process::exit(1);
        }
    }
}
