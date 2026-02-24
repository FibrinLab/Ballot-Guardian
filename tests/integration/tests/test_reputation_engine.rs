#[macro_use]
#[path = "helpers.rs"]
mod helpers;

use borsh::BorshDeserialize;
use helpers::TestEnv;
use reputation_engine::{
    ApplyComponentDeltaArgs, InitializeRealmConfigArgs, RealmReputationConfig,
    ReputationInstruction, ReputationProfile,
};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_program,
};

// ── Instruction builders ────────────────────────────────────────────

fn ix_init_config(pid: &Pubkey, admin: &Pubkey, args: InitializeRealmConfigArgs) -> Instruction {
    let (config, _) = helpers::reputation_config_pda(pid, &args.realm);
    Instruction::new_with_borsh(
        *pid,
        &ReputationInstruction::InitializeRealmConfig(args),
        vec![
            AccountMeta::new(*admin, true),
            AccountMeta::new(config, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    )
}

fn ix_set_oracle(
    pid: &Pubkey,
    admin: &Pubkey,
    config: &Pubkey,
    new_oracle: Pubkey,
) -> Instruction {
    Instruction::new_with_borsh(
        *pid,
        &ReputationInstruction::SetOracleAuthority {
            new_oracle_authority: new_oracle,
        },
        vec![
            AccountMeta::new_readonly(*admin, true),
            AccountMeta::new(*config, false),
        ],
    )
}

fn ix_create_profile(
    pid: &Pubkey,
    payer: &Pubkey,
    config: &Pubkey,
    member: &Pubkey,
    realm: &Pubkey,
) -> Instruction {
    let (profile, _) = helpers::reputation_profile_pda(pid, realm, member);
    Instruction::new_with_borsh(
        *pid,
        &ReputationInstruction::CreateProfile,
        vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new_readonly(*member, false),
            AccountMeta::new(profile, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    )
}

fn ix_apply_delta(
    pid: &Pubkey,
    updater: &Pubkey,
    config: &Pubkey,
    member: &Pubkey,
    realm: &Pubkey,
    args: ApplyComponentDeltaArgs,
) -> Instruction {
    let (profile, _) = helpers::reputation_profile_pda(pid, realm, member);
    Instruction::new_with_borsh(
        *pid,
        &ReputationInstruction::ApplyComponentDelta(args),
        vec![
            AccountMeta::new_readonly(*updater, true),
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new_readonly(*member, false),
            AccountMeta::new(profile, false),
        ],
    )
}

fn ix_apply_penalty(
    pid: &Pubkey,
    updater: &Pubkey,
    config: &Pubkey,
    member: &Pubkey,
    realm: &Pubkey,
    penalty_points: u32,
    reason_code: u16,
) -> Instruction {
    let (profile, _) = helpers::reputation_profile_pda(pid, realm, member);
    Instruction::new_with_borsh(
        *pid,
        &ReputationInstruction::ApplyPenalty {
            penalty_points,
            reason_code,
        },
        vec![
            AccountMeta::new_readonly(*updater, true),
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new_readonly(*member, false),
            AccountMeta::new(profile, false),
        ],
    )
}

fn ix_recalculate(
    pid: &Pubkey,
    updater: &Pubkey,
    config: &Pubkey,
    member: &Pubkey,
    realm: &Pubkey,
) -> Instruction {
    let (profile, _) = helpers::reputation_profile_pda(pid, realm, member);
    Instruction::new_with_borsh(
        *pid,
        &ReputationInstruction::RecalculateProfile,
        vec![
            AccountMeta::new_readonly(*updater, true),
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new_readonly(*member, false),
            AccountMeta::new(profile, false),
        ],
    )
}

fn ix_snapshot(pid: &Pubkey, config: &Pubkey, member: &Pubkey, realm: &Pubkey) -> Instruction {
    let (profile, _) = helpers::reputation_profile_pda(pid, realm, member);
    Instruction::new_with_borsh(
        *pid,
        &ReputationInstruction::SnapshotMultiplier,
        vec![
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new_readonly(*member, false),
            AccountMeta::new_readonly(profile, false),
        ],
    )
}

// ── Default arguments ───────────────────────────────────────────────

fn default_config_args(realm: Pubkey) -> InitializeRealmConfigArgs {
    InitializeRealmConfigArgs {
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
    }
}

/// Initialize config + create a profile. Returns (config_pda, member_keypair, realm).
fn setup_with_profile(env: &mut TestEnv) -> (Pubkey, Keypair, Pubkey) {
    let realm = Pubkey::new_unique();
    let (config, _) = helpers::reputation_config_pda(&env.rep_pid, &realm);
    let member = Keypair::new();

    let ix1 = ix_init_config(&env.rep_pid, &env.payer.pubkey(), default_config_args(realm));
    helpers::send_tx(&mut env.svm, &[ix1], &env.payer, &[&env.payer]).unwrap();

    let ix2 = ix_create_profile(
        &env.rep_pid,
        &env.payer.pubkey(),
        &config,
        &member.pubkey(),
        &realm,
    );
    helpers::send_tx(&mut env.svm, &[ix2], &env.payer, &[&env.payer]).unwrap();

    (config, member, realm)
}

// ═══════════════════════════════════════════════════════════════════
// Happy-path tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_init_config() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let (config, _) = helpers::reputation_config_pda(&env.rep_pid, &realm);

    let ix = ix_init_config(&env.rep_pid, &env.payer.pubkey(), default_config_args(realm));
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    assert!(result.is_ok(), "{:?}", result.err());

    let account = env.svm.get_account(&config).expect("config not found");
    let data = RealmReputationConfig::try_from_slice(&account.data).unwrap();
    assert_eq!(data.realm, realm);
    assert_eq!(data.admin, env.payer.pubkey());
    assert_eq!(data.oracle_authority, env.payer.pubkey()); // defaults to admin
    assert_eq!(data.min_multiplier_bps, 5_000);
    assert_eq!(data.base_multiplier_bps, 10_000);
    assert_eq!(data.max_multiplier_bps, 20_000);
    assert_eq!(data.participation_weight, 10);
    assert_eq!(data.points_per_bonus_bps, 100);
}

#[test]
fn test_set_oracle() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let (config, _) = helpers::reputation_config_pda(&env.rep_pid, &realm);

    let ix = ix_init_config(&env.rep_pid, &env.payer.pubkey(), default_config_args(realm));
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    let new_oracle = Pubkey::new_unique();
    let ix2 = ix_set_oracle(&env.rep_pid, &env.payer.pubkey(), &config, new_oracle);
    let result = helpers::send_tx(&mut env.svm, &[ix2], &env.payer, &[&env.payer]);
    assert!(result.is_ok(), "{:?}", result.err());

    let account = env.svm.get_account(&config).unwrap();
    let data = RealmReputationConfig::try_from_slice(&account.data).unwrap();
    assert_eq!(data.oracle_authority, new_oracle);
}

#[test]
fn test_create_profile() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let (config, _) = helpers::reputation_config_pda(&env.rep_pid, &realm);
    let member = Pubkey::new_unique();

    let ix = ix_init_config(&env.rep_pid, &env.payer.pubkey(), default_config_args(realm));
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    let ix2 = ix_create_profile(
        &env.rep_pid,
        &env.payer.pubkey(),
        &config,
        &member,
        &realm,
    );
    let result = helpers::send_tx(&mut env.svm, &[ix2], &env.payer, &[&env.payer]);
    assert!(result.is_ok(), "{:?}", result.err());

    let (profile, _) = helpers::reputation_profile_pda(&env.rep_pid, &realm, &member);
    let account = env.svm.get_account(&profile).unwrap();
    let data = ReputationProfile::try_from_slice(&account.data).unwrap();
    assert_eq!(data.realm, realm);
    assert_eq!(data.member, member);
    assert_eq!(data.multiplier_bps, 10_000); // base_multiplier_bps
    assert_eq!(data.participation_score, 0);
    assert_eq!(data.penalties_score, 0);
}

#[test]
fn test_apply_delta() {
    require_sbf!();
    let mut env = helpers::setup();
    let (config, member, realm) = setup_with_profile(&mut env);

    let args = ApplyComponentDeltaArgs {
        participation_delta: 50,
        proposal_delta: 30,
        staking_delta: 20,
        tenure_delta: 10,
        delegation_delta: 5,
    };
    let ix = ix_apply_delta(
        &env.rep_pid,
        &env.payer.pubkey(),
        &config,
        &member.pubkey(),
        &realm,
        args,
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    assert!(result.is_ok(), "{:?}", result.err());

    let (profile_key, _) = helpers::reputation_profile_pda(&env.rep_pid, &realm, &member.pubkey());
    let account = env.svm.get_account(&profile_key).unwrap();
    let data = ReputationProfile::try_from_slice(&account.data).unwrap();
    assert_eq!(data.participation_score, 50);
    assert_eq!(data.proposal_creation_score, 30);
    assert_eq!(data.staking_score, 20);
    assert_eq!(data.tenure_score, 10);
    assert_eq!(data.delegation_trust_score, 5);
}

#[test]
fn test_apply_penalty() {
    require_sbf!();
    let mut env = helpers::setup();
    let (config, member, realm) = setup_with_profile(&mut env);

    let ix = ix_apply_penalty(
        &env.rep_pid,
        &env.payer.pubkey(),
        &config,
        &member.pubkey(),
        &realm,
        5,
        1001,
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    assert!(result.is_ok(), "{:?}", result.err());

    let (profile_key, _) = helpers::reputation_profile_pda(&env.rep_pid, &realm, &member.pubkey());
    let account = env.svm.get_account(&profile_key).unwrap();
    let data = ReputationProfile::try_from_slice(&account.data).unwrap();
    assert_eq!(data.penalties_score, 5);
    // base(10000) - penalty(5 * 50 = 250) = 9750
    assert_eq!(data.multiplier_bps, 9_750);
}

#[test]
fn test_recalculate() {
    require_sbf!();
    let mut env = helpers::setup();
    let (config, member, realm) = setup_with_profile(&mut env);

    let ix = ix_recalculate(
        &env.rep_pid,
        &env.payer.pubkey(),
        &config,
        &member.pubkey(),
        &realm,
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn test_snapshot() {
    require_sbf!();
    let mut env = helpers::setup();
    let (config, member, realm) = setup_with_profile(&mut env);

    let ix = ix_snapshot(&env.rep_pid, &config, &member.pubkey(), &realm);
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn test_full_lifecycle() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let (config, _) = helpers::reputation_config_pda(&env.rep_pid, &realm);
    let member = Keypair::new();
    let oracle = Keypair::new();
    env.svm
        .airdrop(&oracle.pubkey(), 1_000_000_000)
        .unwrap();

    // 1. Init config
    let ix = ix_init_config(&env.rep_pid, &env.payer.pubkey(), default_config_args(realm));
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    // 2. Set oracle
    let ix = ix_set_oracle(&env.rep_pid, &env.payer.pubkey(), &config, oracle.pubkey());
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    // 3. Create profile
    let ix = ix_create_profile(
        &env.rep_pid,
        &env.payer.pubkey(),
        &config,
        &member.pubkey(),
        &realm,
    );
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    // 4. Oracle applies delta
    let args = ApplyComponentDeltaArgs {
        participation_delta: 100,
        proposal_delta: 50,
        staking_delta: 25,
        tenure_delta: 10,
        delegation_delta: 5,
    };
    let ix = ix_apply_delta(
        &env.rep_pid,
        &oracle.pubkey(),
        &config,
        &member.pubkey(),
        &realm,
        args,
    );
    helpers::send_tx(&mut env.svm, &[ix], &oracle, &[&oracle]).unwrap();

    // 5. Apply penalty
    let ix = ix_apply_penalty(
        &env.rep_pid,
        &oracle.pubkey(),
        &config,
        &member.pubkey(),
        &realm,
        2,
        42,
    );
    helpers::send_tx(&mut env.svm, &[ix], &oracle, &[&oracle]).unwrap();

    // 6. Recalculate
    let ix = ix_recalculate(
        &env.rep_pid,
        &oracle.pubkey(),
        &config,
        &member.pubkey(),
        &realm,
    );
    helpers::send_tx(&mut env.svm, &[ix], &oracle, &[&oracle]).unwrap();

    // 7. Snapshot
    let ix = ix_snapshot(&env.rep_pid, &config, &member.pubkey(), &realm);
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    // Verify final state
    let (profile_key, _) = helpers::reputation_profile_pda(&env.rep_pid, &realm, &member.pubkey());
    let account = env.svm.get_account(&profile_key).unwrap();
    let profile = ReputationProfile::try_from_slice(&account.data).unwrap();
    assert_eq!(profile.participation_score, 100);
    assert_eq!(profile.proposal_creation_score, 50);
    assert_eq!(profile.staking_score, 25);
    assert_eq!(profile.tenure_score, 10);
    assert_eq!(profile.delegation_trust_score, 5);
    assert_eq!(profile.penalties_score, 2);
    // weighted_points = 100*10 + 50*20 + 25*15 + 10*5 + 5*10 = 1000+1000+375+50+50 = 2475
    // bonus = 2475 / 100 = 24 bps (< max_bonus_bps 5000)
    // penalty = 2 * 50 = 100 bps (< max_penalty_bps 5000)
    // multiplier = 10000 + 24 - 100 = 9924
    assert_eq!(profile.multiplier_bps, 9_924);
}

// ═══════════════════════════════════════════════════════════════════
// Error-case tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_invalid_multiplier_range() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();

    // min > base
    let mut args = default_config_args(realm);
    args.min_multiplier_bps = 15_000;
    args.base_multiplier_bps = 10_000;

    let ix = ix_init_config(&env.rep_pid, &env.payer.pubkey(), args);
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    helpers::assert_custom_error(&result, 1); // InvalidMultiplierConfig
}

#[test]
fn test_min_too_low() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();

    let mut args = default_config_args(realm);
    args.min_multiplier_bps = 4_999; // below 5000

    let ix = ix_init_config(&env.rep_pid, &env.payer.pubkey(), args);
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    helpers::assert_custom_error(&result, 1); // InvalidMultiplierConfig
}

#[test]
fn test_max_too_high() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();

    let mut args = default_config_args(realm);
    args.max_multiplier_bps = 20_001; // above 20000

    let ix = ix_init_config(&env.rep_pid, &env.payer.pubkey(), args);
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    helpers::assert_custom_error(&result, 1); // InvalidMultiplierConfig
}

#[test]
fn test_zero_scoring_params() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();

    let mut args = default_config_args(realm);
    args.points_per_bonus_bps = 0;

    let ix = ix_init_config(&env.rep_pid, &env.payer.pubkey(), args);
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    helpers::assert_custom_error(&result, 2); // InvalidScoringConfig
}

#[test]
fn test_unauthorized_set_oracle() {
    require_sbf!();
    let mut env = helpers::setup();
    let realm = Pubkey::new_unique();
    let (config, _) = helpers::reputation_config_pda(&env.rep_pid, &realm);

    let ix = ix_init_config(&env.rep_pid, &env.payer.pubkey(), default_config_args(realm));
    helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]).unwrap();

    let impostor = Keypair::new();
    env.svm
        .airdrop(&impostor.pubkey(), 1_000_000_000)
        .unwrap();
    let ix = ix_set_oracle(
        &env.rep_pid,
        &impostor.pubkey(),
        &config,
        Pubkey::new_unique(),
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &impostor, &[&impostor]);
    helpers::assert_custom_error(&result, 0); // Unauthorized
}

#[test]
fn test_unauthorized_delta() {
    require_sbf!();
    let mut env = helpers::setup();
    let (config, member, realm) = setup_with_profile(&mut env);

    let impostor = Keypair::new();
    env.svm
        .airdrop(&impostor.pubkey(), 1_000_000_000)
        .unwrap();
    let args = ApplyComponentDeltaArgs {
        participation_delta: 10,
        ..Default::default()
    };
    let ix = ix_apply_delta(
        &env.rep_pid,
        &impostor.pubkey(),
        &config,
        &member.pubkey(),
        &realm,
        args,
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &impostor, &[&impostor]);
    helpers::assert_custom_error(&result, 0); // Unauthorized
}

#[test]
fn test_zero_penalty_points() {
    require_sbf!();
    let mut env = helpers::setup();
    let (config, member, realm) = setup_with_profile(&mut env);

    let ix = ix_apply_penalty(
        &env.rep_pid,
        &env.payer.pubkey(),
        &config,
        &member.pubkey(),
        &realm,
        0, // zero penalty
        0,
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    helpers::assert_custom_error(&result, 4); // InvalidPenalty
}

#[test]
fn test_wrong_member() {
    require_sbf!();
    let mut env = helpers::setup();
    let (config, _member, realm) = setup_with_profile(&mut env);

    // Use a different member key — the derived PDA won't exist as an account
    let wrong_member = Pubkey::new_unique();
    let args = ApplyComponentDeltaArgs {
        participation_delta: 10,
        ..Default::default()
    };
    let ix = ix_apply_delta(
        &env.rep_pid,
        &env.payer.pubkey(),
        &config,
        &wrong_member,
        &realm,
        args,
    );
    let result = helpers::send_tx(&mut env.svm, &[ix], &env.payer, &[&env.payer]);
    helpers::assert_tx_err(&result);
}
