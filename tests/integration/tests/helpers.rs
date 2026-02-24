#![allow(dead_code)]

use litesvm::LiteSVM;
use solana_sdk::{
    instruction::Instruction,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};
use std::path::{Path, PathBuf};

// ── .so file discovery ──────────────────────────────────────────────

fn so_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy")
}

pub fn programs_built() -> bool {
    let dir = so_dir();
    dir.join("reputation_engine.so").exists()
        && dir.join("quadratic_voting.so").exists()
        && dir.join("realms_adapter.so").exists()
}

#[allow(unused_macros)]
macro_rules! require_sbf {
    () => {
        if !helpers::programs_built() {
            eprintln!(
                "Skipping test: .so files not found in target/deploy/. Run `cargo build-sbf` first."
            );
            return;
        }
    };
}
#[allow(unused_imports)]
pub(crate) use require_sbf;

// ── Test environment ────────────────────────────────────────────────

pub struct TestEnv {
    pub svm: LiteSVM,
    pub payer: Keypair,
    pub rep_pid: Pubkey,
    pub qv_pid: Pubkey,
    pub adapter_pid: Pubkey,
}

pub fn setup() -> TestEnv {
    let mut svm = LiteSVM::new();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 100 * LAMPORTS_PER_SOL)
        .unwrap();

    let dir = so_dir();
    let rep_pid = Pubkey::new_unique();
    let qv_pid = Pubkey::new_unique();
    let adapter_pid = Pubkey::new_unique();

    svm.add_program(
        rep_pid,
        &std::fs::read(dir.join("reputation_engine.so")).unwrap(),
    );
    svm.add_program(
        qv_pid,
        &std::fs::read(dir.join("quadratic_voting.so")).unwrap(),
    );
    svm.add_program(
        adapter_pid,
        &std::fs::read(dir.join("realms_adapter.so")).unwrap(),
    );

    TestEnv {
        svm,
        payer,
        rep_pid,
        qv_pid,
        adapter_pid,
    }
}

// ── Transaction helpers ─────────────────────────────────────────────

pub fn send_tx(
    svm: &mut LiteSVM,
    ixs: &[Instruction],
    payer: &Keypair,
    signers: &[&Keypair],
) -> Result<litesvm::types::TransactionMetadata, litesvm::types::FailedTransactionMetadata> {
    let blockhash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(ixs, Some(&payer.pubkey()), signers, blockhash);
    svm.send_transaction(tx)
}

// ── Error assertion helpers ─────────────────────────────────────────

/// Assert a transaction failed with a specific custom program error code.
pub fn assert_custom_error<S: std::fmt::Debug, E: std::fmt::Debug>(
    result: &Result<S, E>,
    expected_code: u32,
) {
    assert!(result.is_err(), "Expected error, got success");
    let err = format!("{:?}", result.as_ref().unwrap_err());
    let expected = format!("Custom({})", expected_code);
    assert!(
        err.contains(&expected),
        "Expected Custom({}) in error, got: {}",
        expected_code,
        err,
    );
}

/// Assert a transaction failed (any error).
pub fn assert_tx_err<S: std::fmt::Debug, E: std::fmt::Debug>(result: &Result<S, E>) {
    assert!(
        result.is_err(),
        "Expected error, got: {:?}",
        result.as_ref().unwrap()
    );
}

// ── PDA derivation helpers ──────────────────────────────────────────

// Reputation engine
pub fn reputation_config_pda(program_id: &Pubkey, realm: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"reputation-config", realm.as_ref()], program_id)
}

pub fn reputation_profile_pda(
    program_id: &Pubkey,
    realm: &Pubkey,
    member: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"reputation-profile", realm.as_ref(), member.as_ref()],
        program_id,
    )
}

// Quadratic voting
pub fn ballot_pda(program_id: &Pubkey, realm: &Pubkey, proposal: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"ballot", realm.as_ref(), proposal.as_ref()],
        program_id,
    )
}

pub fn allocation_pda(program_id: &Pubkey, ballot: &Pubkey, voter: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"allocation", ballot.as_ref(), voter.as_ref()],
        program_id,
    )
}

// Realms adapter
pub fn adapter_config_pda(program_id: &Pubkey, realm: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"adapter-config", realm.as_ref()], program_id)
}

pub fn proposal_binding_pda(
    program_id: &Pubkey,
    realm: &Pubkey,
    proposal: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"proposal-binding", realm.as_ref(), proposal.as_ref()],
        program_id,
    )
}

pub fn voter_weight_pda(program_id: &Pubkey, binding: &Pubkey, voter: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"voter-weight", binding.as_ref(), voter.as_ref()],
        program_id,
    )
}
