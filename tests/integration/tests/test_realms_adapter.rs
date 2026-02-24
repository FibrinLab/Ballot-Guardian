#[macro_use]
#[path = "helpers.rs"]
mod helpers;

use borsh::BorshDeserialize;
use helpers::TestEnv;
use realms_adapter::{
    AdapterConfig, BindProposalArgs, InitializeAdapterArgs, PluginVoterWeightRecord,
    ProposalBinding, RealmsAdapterInstruction, RefreshVoterWeightArgs,
};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_program,
};

// ── Instruction builders ────────────────────────────────────────────

fn ix_init_adapter(pid: &Pubkey, admin: &Pubkey, args: InitializeAdapterArgs) -> Instruction {
    let (config, _) = helpers::adapter_config_pda(pid, &args.realm);
    Instruction::new_with_borsh(
        *pid,
        &RealmsAdapterInstruction::InitializeAdapter(args),
        vec![
            AccountMeta::new(*admin, true),
            AccountMeta::new(config, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    )
}

fn ix_bind_proposal(
    pid: &Pubkey,
    admin: &Pubkey,
    config: &Pubkey,
    realm: &Pubkey,
    args: BindProposalArgs,
) -> Instruction {
    let (binding, _) = helpers::proposal_binding_pda(pid, realm, &args.proposal);
    Instruction::new_with_borsh(
        *pid,
        &RealmsAdapterInstruction::BindProposal(args),
        vec![
            AccountMeta::new(*admin, true),
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new(binding, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    )
}

fn ix_set_council_override(
    pid: &Pubkey,
    authority: &Pubkey,
    config: &Pubkey,
    proposal: &Pubkey,
    binding: &Pubkey,
    active: bool,
    reason_code: u16,
) -> Instruction {
    Instruction::new_with_borsh(
        *pid,
        &RealmsAdapterInstruction::SetCouncilOverride {
            active,
            reason_code,
        },
        vec![
            AccountMeta::new_readonly(*authority, true),
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new_readonly(*proposal, false),
            AccountMeta::new(*binding, false),
        ],
    )
}

fn ix_create_voter_weight_record(
    pid: &Pubkey,
    payer: &Pubkey,
    config: &Pubkey,
    proposal: &Pubkey,
    binding: &Pubkey,
    voter: &Pubkey,
) -> Instruction {
    let (record, _) = helpers::voter_weight_pda(pid, binding, voter);
    Instruction::new_with_borsh(
        *pid,
        &RealmsAdapterInstruction::CreateVoterWeightRecord,
        vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new_readonly(*proposal, false),
            AccountMeta::new_readonly(*binding, false),
            AccountMeta::new_readonly(*voter, false),
            AccountMeta::new(record, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    )
}

fn ix_refresh_voter_weight(
    pid: &Pubkey,
    caller: &Pubkey,
    config: &Pubkey,
    proposal: &Pubkey,
    binding: &Pubkey,
    voter: &Pubkey,
    args: RefreshVoterWeightArgs,
) -> Instruction {
    let (record, _) = helpers::voter_weight_pda(pid, binding, voter);
    Instruction::new_with_borsh(
        *pid,
        &RealmsAdapterInstruction::RefreshVoterWeightRecord(args),
        vec![
            AccountMeta::new_readonly(*caller, true),
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new_readonly(*proposal, false),
            AccountMeta::new(*binding, false),
            AccountMeta::new_readonly(*voter, false),
            AccountMeta::new(record, false),
        ],
    )
}

// ── Default arguments ───────────────────────────────────────────────

fn default_adapter_args(realm: Pubkey) -> InitializeAdapterArgs {
    InitializeAdapterArgs {
        realm,
        governance_program_id: Pubkey::new_unique(),
        quadratic_voting_program: Pubkey::new_unique(),
        reputation_engine_program: Pubkey::new_unique(),
        council_override_authority: None,
        min_reputation_bps: 5_000,
        max_reputation_bps: 20_000,
    }
}

fn default_bind_args(proposal: Pubkey) -> BindProposalArgs {
    BindProposalArgs {
        proposal,
        quadratic_ballot: Pubkey::new_unique(),
        governing_token_mint: Pubkey::new_unique(),
        council_override_enabled: true,
        default_weight_expiry_slot: 1000,
    }
}

/// Setup adapter config + binding + voter weight record.
/// Returns (config, proposal, binding, voter, governing_token_mint, realm).
fn setup_full(env: &mut TestEnv) -> (Pubkey, Pubkey, Pubkey, Pubkey, Pubkey, Pubkey) {
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let (config, _) = helpers::adapter_config_pda(&env.adapter_pid, &realm);

    let ix = ix_init_adapter(
        &env.adapter_pid,
        &env.payer.pubkey(),
        default_adapter_args(realm),
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    let bind_args = default_bind_args(proposal);
    let governing_token_mint = bind_args.governing_token_mint;
    let (binding, _) = helpers::proposal_binding_pda(&env.adapter_pid, &realm, &proposal);

    let ix = ix_bind_proposal(
        &env.adapter_pid,
        &env.payer.pubkey(),
        &config,
        &realm,
        bind_args,
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    let voter = Pubkey::new_unique();
    let ix = ix_create_voter_weight_record(
        &env.adapter_pid,
        &env.payer.pubkey(),
        &config,
        &proposal,
        &binding,
        &voter,
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    (config, proposal, binding, voter, governing_token_mint, realm)
}

// ═══════════════════════════════════════════════════════════════════
// Happy-path tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_init_adapter() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let (config, _) = helpers::adapter_config_pda(&env.adapter_pid, &realm);

    let ix = ix_init_adapter(
        &env.adapter_pid,
        &env.payer.pubkey(),
        default_adapter_args(realm),
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    assert!(result.is_ok(), "{:?}", result.err());

    let account = env.svm.get_account(&config).unwrap();
    let data = AdapterConfig::try_from_slice(&account.data).unwrap();
    assert_eq!(data.realm, realm);
    assert_eq!(data.admin, env.payer.pubkey());
    assert_eq!(data.min_reputation_bps, 5_000);
    assert_eq!(data.max_reputation_bps, 20_000);
}

#[test]
fn test_bind_proposal() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let (config, _) = helpers::adapter_config_pda(&env.adapter_pid, &realm);

    let ix = ix_init_adapter(
        &env.adapter_pid,
        &env.payer.pubkey(),
        default_adapter_args(realm),
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    let bind_args = default_bind_args(proposal);
    let qv_ballot = bind_args.quadratic_ballot;
    let (binding, _) = helpers::proposal_binding_pda(&env.adapter_pid, &realm, &proposal);

    let ix = ix_bind_proposal(
        &env.adapter_pid,
        &env.payer.pubkey(),
        &config,
        &realm,
        bind_args,
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    assert!(result.is_ok(), "{:?}", result.err());

    let account = env.svm.get_account(&binding).unwrap();
    let data = ProposalBinding::try_from_slice(&account.data).unwrap();
    assert_eq!(data.proposal, proposal);
    assert_eq!(data.quadratic_ballot, qv_ballot);
    assert!(data.council_override_enabled);
    assert!(!data.council_override_active);
}

#[test]
fn test_set_council_override_on() {
    require_sbf!();
    let mut env = helpers::setup();
    let (config, proposal, binding, _voter, _mint, _realm) = setup_full(&mut env);

    let ix = ix_set_council_override(
        &env.adapter_pid,
        &env.payer.pubkey(),
        &config,
        &proposal,
        &binding,
        true,
        42,
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    assert!(result.is_ok(), "{:?}", result.err());

    let account = env.svm.get_account(&binding).unwrap();
    let data = ProposalBinding::try_from_slice(&account.data).unwrap();
    assert!(data.council_override_active);
    assert_eq!(data.council_override_reason_code, 42);
}

#[test]
fn test_set_council_override_off() {
    require_sbf!();
    let mut env = helpers::setup();
    let (config, proposal, binding, _voter, _mint, _realm) = setup_full(&mut env);

    // Turn on
    let ix = ix_set_council_override(
        &env.adapter_pid,
        &env.payer.pubkey(),
        &config,
        &proposal,
        &binding,
        true,
        42,
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    // Turn off
    let ix = ix_set_council_override(
        &env.adapter_pid,
        &env.payer.pubkey(),
        &config,
        &proposal,
        &binding,
        false,
        0,
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    assert!(result.is_ok(), "{:?}", result.err());

    let account = env.svm.get_account(&binding).unwrap();
    let data = ProposalBinding::try_from_slice(&account.data).unwrap();
    assert!(!data.council_override_active);
}

#[test]
fn test_create_voter_weight_record() {
    require_sbf!();
    let mut env = helpers::setup();
    let (_config, _proposal, binding, voter, mint, _realm) = setup_full(&mut env);

    let (record, _) = helpers::voter_weight_pda(&env.adapter_pid, &binding, &voter);
    let account = env.svm.get_account(&record).unwrap();
    let data = PluginVoterWeightRecord::deserialize(&mut &account.data[..]).unwrap();
    assert_eq!(data.governing_token_owner, voter);
    assert_eq!(data.governing_token_mint, mint);
    assert_eq!(data.voter_weight, 0);
    assert_eq!(data.token_amount_allocated, 0);
    assert_eq!(data.qv_votes_allocated, 0);
}

#[test]
fn test_refresh_record() {
    require_sbf!();
    let mut env = helpers::setup();
    let (config, proposal, binding, voter, _mint, _realm) = setup_full(&mut env);

    let args = RefreshVoterWeightArgs {
        token_amount_allocated: 10_000,
        qv_votes_allocated: 0,
        reputation_multiplier_bps: 10_000,
        voter_weight_expiry: None,
        weight_action: None,
        weight_action_target: None,
    };
    let ix = ix_refresh_voter_weight(
        &env.adapter_pid,
        &env.payer.pubkey(),
        &config,
        &proposal,
        &binding,
        &voter,
        args,
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    assert!(result.is_ok(), "{:?}", result.err());

    let (record, _) = helpers::voter_weight_pda(&env.adapter_pid, &binding, &voter);
    let account = env.svm.get_account(&record).unwrap();
    let data = PluginVoterWeightRecord::deserialize(&mut &account.data[..]).unwrap();
    // qv_votes_allocated == 0, so qv_component = sqrt(10000) = 100
    // effective_weight = 100 * 10000 / 10000 = 100
    assert_eq!(data.voter_weight, 100);
    assert_eq!(data.token_amount_allocated, 10_000);
}

#[test]
fn test_refresh_with_qv_votes_override() {
    require_sbf!();
    let mut env = helpers::setup();
    let (config, proposal, binding, voter, _mint, _realm) = setup_full(&mut env);

    let args = RefreshVoterWeightArgs {
        token_amount_allocated: 10_000,
        qv_votes_allocated: 50, // qv_votes > 0 overrides sqrt
        reputation_multiplier_bps: 15_000,
        voter_weight_expiry: Some(999),
        weight_action: None,
        weight_action_target: None,
    };
    let ix = ix_refresh_voter_weight(
        &env.adapter_pid,
        &env.payer.pubkey(),
        &config,
        &proposal,
        &binding,
        &voter,
        args,
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    assert!(result.is_ok(), "{:?}", result.err());

    let (record, _) = helpers::voter_weight_pda(&env.adapter_pid, &binding, &voter);
    let account = env.svm.get_account(&record).unwrap();
    let data = PluginVoterWeightRecord::deserialize(&mut &account.data[..]).unwrap();
    // qv_component = 50 (from qv_votes_allocated)
    // effective_weight = 50 * 15000 / 10000 = 75
    assert_eq!(data.voter_weight, 75);
    assert_eq!(data.qv_votes_allocated, 50);
    assert_eq!(data.voter_weight_expiry, Some(999));
}

#[test]
fn test_full_lifecycle() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let (config, _) = helpers::adapter_config_pda(&env.adapter_pid, &realm);

    // 1. Init adapter
    let ix = ix_init_adapter(
        &env.adapter_pid,
        &env.payer.pubkey(),
        default_adapter_args(realm),
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    // 2. Bind proposal
    let bind_args = default_bind_args(proposal);
    let (binding, _) = helpers::proposal_binding_pda(&env.adapter_pid, &realm, &proposal);
    let ix = ix_bind_proposal(
        &env.adapter_pid,
        &env.payer.pubkey(),
        &config,
        &realm,
        bind_args,
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    // 3. Council override on
    let ix = ix_set_council_override(
        &env.adapter_pid,
        &env.payer.pubkey(),
        &config,
        &proposal,
        &binding,
        true,
        1,
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    // 4. Create voter weight record
    let voter = Pubkey::new_unique();
    let ix = ix_create_voter_weight_record(
        &env.adapter_pid,
        &env.payer.pubkey(),
        &config,
        &proposal,
        &binding,
        &voter,
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    // 5. Refresh with data
    let args = RefreshVoterWeightArgs {
        token_amount_allocated: 10_000,
        qv_votes_allocated: 25,
        reputation_multiplier_bps: 12_000,
        voter_weight_expiry: Some(500),
        weight_action: None,
        weight_action_target: None,
    };
    let ix = ix_refresh_voter_weight(
        &env.adapter_pid,
        &env.payer.pubkey(),
        &config,
        &proposal,
        &binding,
        &voter,
        args,
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    // Verify
    let (record, _) = helpers::voter_weight_pda(&env.adapter_pid, &binding, &voter);
    let account = env.svm.get_account(&record).unwrap();
    let data = PluginVoterWeightRecord::deserialize(&mut &account.data[..]).unwrap();
    // qv_component = 25, weight = 25 * 12000 / 10000 = 30
    assert_eq!(data.voter_weight, 30);
    assert!(data.council_override_active);
    assert_eq!(data.voter_weight_expiry, Some(500));
}

// ═══════════════════════════════════════════════════════════════════
// Error-case tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_invalid_rep_bounds() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();

    let mut args = default_adapter_args(realm);
    args.min_reputation_bps = 0; // invalid — must be > 0

    let ix = ix_init_adapter(&env.adapter_pid, &env.payer.pubkey(), args);
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    helpers::assert_custom_error(&result, 1); // InvalidReputationBounds
}

#[test]
fn test_unauthorized_bind() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let (config, _) = helpers::adapter_config_pda(&env.adapter_pid, &realm);

    let ix = ix_init_adapter(
        &env.adapter_pid,
        &env.payer.pubkey(),
        default_adapter_args(realm),
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    let impostor = Keypair::new();
    env.svm
        .airdrop(&impostor.pubkey(), 1_000_000_000)
        .unwrap();

    let ix = ix_bind_proposal(
        &env.adapter_pid,
        &impostor.pubkey(),
        &config,
        &realm,
        default_bind_args(proposal),
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &impostor, &[&impostor]);
    helpers::assert_custom_error(&result, 0); // Unauthorized
}

#[test]
fn test_council_override_disabled() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let proposal = Pubkey::new_unique();
    let (config, _) = helpers::adapter_config_pda(&env.adapter_pid, &realm);

    let ix = ix_init_adapter(
        &env.adapter_pid,
        &env.payer.pubkey(),
        default_adapter_args(realm),
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    // Bind with council_override_enabled = false
    let mut bind_args = default_bind_args(proposal);
    bind_args.council_override_enabled = false;
    let (binding, _) = helpers::proposal_binding_pda(&env.adapter_pid, &realm, &proposal);

    let ix = ix_bind_proposal(
        &env.adapter_pid,
        &env.payer.pubkey(),
        &config,
        &realm,
        bind_args,
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    // Try to set override
    let ix = ix_set_council_override(
        &env.adapter_pid,
        &env.payer.pubkey(),
        &config,
        &proposal,
        &binding,
        true,
        1,
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    helpers::assert_custom_error(&result, 8); // CouncilOverrideDisabled
}

#[test]
fn test_unauthorized_override() {
    require_sbf!();
    let mut env = helpers::setup();
    let (config, proposal, binding, _voter, _mint, _realm) = setup_full(&mut env);

    let impostor = Keypair::new();
    env.svm
        .airdrop(&impostor.pubkey(), 1_000_000_000)
        .unwrap();

    let ix = ix_set_council_override(
        &env.adapter_pid,
        &impostor.pubkey(),
        &config,
        &proposal,
        &binding,
        true,
        1,
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &impostor, &[&impostor]);
    helpers::assert_custom_error(&result, 0); // Unauthorized
}

#[test]
fn test_rep_out_of_bounds() {
    require_sbf!();
    let mut env = helpers::setup();
    let (config, proposal, binding, voter, _mint, _realm) = setup_full(&mut env);

    // Multiplier below adapter min
    let args = RefreshVoterWeightArgs {
        token_amount_allocated: 1000,
        qv_votes_allocated: 0,
        reputation_multiplier_bps: 4_000, // below min 5000
        voter_weight_expiry: None,
        weight_action: None,
        weight_action_target: None,
    };
    let ix = ix_refresh_voter_weight(
        &env.adapter_pid,
        &env.payer.pubkey(),
        &config,
        &proposal,
        &binding,
        &voter,
        args,
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    helpers::assert_custom_error(&result, 1); // InvalidReputationBounds
}

#[test]
fn test_voter_mismatch() {
    require_sbf!();
    let mut env = helpers::setup();
    let (config, proposal, binding, voter, _mint, _realm) = setup_full(&mut env);

    // Manually build instruction with wrong voter key in accounts but correct record PDA
    // This creates a mismatch between the voter account and the record's voter field
    let wrong_voter = Pubkey::new_unique();
    let (record, _) = helpers::voter_weight_pda(&env.adapter_pid, &binding, &voter);

    let args = RefreshVoterWeightArgs {
        token_amount_allocated: 1000,
        qv_votes_allocated: 0,
        reputation_multiplier_bps: 10_000,
        voter_weight_expiry: None,
        weight_action: None,
        weight_action_target: None,
    };

    // Pass wrong_voter in accounts but the record PDA derived from original voter
    let ix = Instruction::new_with_borsh(
        env.adapter_pid,
        &RealmsAdapterInstruction::RefreshVoterWeightRecord(args),
        vec![
            AccountMeta::new_readonly(env.payer.pubkey(), true),
            AccountMeta::new_readonly(config, false),
            AccountMeta::new_readonly(proposal, false),
            AccountMeta::new(binding, false),
            AccountMeta::new_readonly(wrong_voter, false),
            AccountMeta::new(record, false),
        ],
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    helpers::assert_tx_err(&result);
}

#[test]
fn test_binding_mismatch() {
    require_sbf!();
    let mut env = helpers::setup();
    let (config, _proposal, binding, voter, _mint, _realm) = setup_full(&mut env);

    // Use wrong proposal key to cause binding PDA mismatch
    let wrong_proposal = Pubkey::new_unique();

    let args = RefreshVoterWeightArgs {
        token_amount_allocated: 1000,
        qv_votes_allocated: 0,
        reputation_multiplier_bps: 10_000,
        voter_weight_expiry: None,
        weight_action: None,
        weight_action_target: None,
    };
    let ix = Instruction::new_with_borsh(
        env.adapter_pid,
        &RealmsAdapterInstruction::RefreshVoterWeightRecord(args),
        vec![
            AccountMeta::new_readonly(env.payer.pubkey(), true),
            AccountMeta::new_readonly(config, false),
            AccountMeta::new_readonly(wrong_proposal, false), // wrong proposal
            AccountMeta::new(binding, false),
            AccountMeta::new_readonly(voter, false),
            AccountMeta::new(
                helpers::voter_weight_pda(&env.adapter_pid, &binding, &voter).0,
                false,
            ),
        ],
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    helpers::assert_tx_err(&result);
}
