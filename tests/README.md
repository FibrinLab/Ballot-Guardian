# Tests

## Overview

Integration tests use [LiteSVM](https://crates.io/crates/litesvm) to load the compiled
`.so` programs and exercise every instruction end-to-end — no running validator needed.

| Crate | Tests | Description |
|---|---|---|
| `ballot-guardian-integration-tests` | 53 | Per-program + cross-program LiteSVM tests |
| `ballot-guardian-examples` | — | Client demo binary (requires local validator) |

## Prerequisites

Install the Solana CLI and build the SBF programs:

```bash
# Install Solana CLI (if not already installed)
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"

# Build .so files
cargo build-sbf
```

After a successful build, the `.so` files will be in `target/deploy/`:
- `reputation_engine.so`
- `quadratic_voting.so`
- `realms_adapter.so`

## Running integration tests

```bash
# Type-check only (no .so files needed):
cargo check -p ballot-guardian-integration-tests

# Full integration test run (requires .so files):
cargo test -p ballot-guardian-integration-tests

# Run a specific test file:
cargo test -p ballot-guardian-integration-tests --test test_reputation_engine
cargo test -p ballot-guardian-integration-tests --test test_quadratic_voting
cargo test -p ballot-guardian-integration-tests --test test_realms_adapter
cargo test -p ballot-guardian-integration-tests --test test_cross_program
```

If the `.so` files are not found, tests will print a skip message and pass (not fail).

## Running the client demo

The client demo connects to a local validator and exercises all three programs:

```bash
# Terminal 1: start the validator with programs loaded
solana-test-validator \
  --bpf-program <REP_PROGRAM_ID> target/deploy/reputation_engine.so \
  --bpf-program <QV_PROGRAM_ID>  target/deploy/quadratic_voting.so \
  --bpf-program <RA_PROGRAM_ID>  target/deploy/realms_adapter.so

# Terminal 2: run the demo
cargo run -p ballot-guardian-examples --bin client_demo
```

Replace `<REP_PROGRAM_ID>`, `<QV_PROGRAM_ID>`, and `<RA_PROGRAM_ID>` with your deployed
program public keys.

## Test structure

```
tests/integration/
  Cargo.toml
  src/lib.rs                         # empty crate root
  tests/
    helpers.rs                       # shared setup (LiteSVM, PDA helpers, assertions)
    test_reputation_engine.rs        # 16 tests (8 happy, 8 error)
    test_quadratic_voting.rs         # 20 tests (7 happy, 13 error)
    test_realms_adapter.rs           # 15 tests (8 happy, 7 error)
    test_cross_program.rs            #  2 cross-program flow tests

examples/
  Cargo.toml
  client_demo.rs                     # localnet client binary
```
