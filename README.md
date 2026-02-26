# Ballot Guardian

**Built by an NHS junior doctor** to fix broken union ballot processes. The BMA represents 190,000+ doctors and still runs strike ballots by post. Ballot Guardian brings wallet-signed, quadratic, reputation-weighted voting to Solana Realms.

```
    +---------------------------+
    |      SPL Governance       |
    |        (Realms)           |
    +------------+--------------+
                 |
                 | reads VoterWeightRecord
                 v
    +---------------------------+
    |     Realms Adapter        |
    |  - AdapterConfig          |
    |  - ProposalBinding        |
    |  - PluginVoterWeightRecord|
    +------+----------+---------+
           |          |
   binds   |          | reads multiplier_bps
   ballot  |          |
           v          v
+----------------+  +--------------------+
| Quadratic      |  | Reputation Engine  |
| Voting         |  |                    |
| - Ballot       |  | - RealmRepConfig   |
| - VoterAlloc   |  | - RepProfile       |
| - cast_vote()  |  | - recompute()      |
+----------------+  +--------------------+
```

---

## Demo

**Judges**: start at `/dapp` -- every action submits a real Solana devnet transaction.

1. **Connect wallet** -- open `/dapp`, connect Phantom (Solana devnet). Ensure you have devnet SOL.
2. **Explore pre-loaded ballots** -- two demo proposals with sample vote tallies (offline data for preview)
3. **Register a new union/DAO** -- fill in name + description, approve the wallet transaction. This creates on-chain `ReputationConfig` + `AdapterConfig` accounts in a single tx.
4. **Create a ballot** -- add a proposal question and voting window. Submits `InitializeBallot` + `BindProposal` on-chain.
5. **Cast a vote** -- select Yes / No / Abstain. The tx auto-registers the voter and creates a Voter Weight Record if needed, then casts the quadratic vote.
6. **View on Explorer** -- every on-chain action shows a link to Solana Explorer. Click "Refresh from chain" to fetch live ballot tallies directly from the program accounts.
7. **View audit trail** -- scroll down to see all wallet-signed actions with cryptographic signatures
8. **View reputation** -- interactive reputation dashboard shows multiplier computation from five scoring components

---

## Deployed Programs (Solana Devnet)

| Program | Address | Instructions |
|---------|---------|-------------|
| **quadratic-voting** | [`346RNEQcBBff4skHhQiRCPe9cDeWpaPTsT2TpQUFYomp`](https://explorer.solana.com/address/346RNEQcBBff4skHhQiRCPe9cDeWpaPTsT2TpQUFYomp?cluster=devnet) | `initialize_ballot`, `register_voter`, `update_voter_budget`, `cast_vote`, `finalize_ballot` |
| **reputation-engine** | [`8JpbKjoR4c7n2HqS51WjyjJrLwvVgGGsKN4o2boohdEA`](https://explorer.solana.com/address/8JpbKjoR4c7n2HqS51WjyjJrLwvVgGGsKN4o2boohdEA?cluster=devnet) | `initialize_realm_config`, `update_realm_config`, `create_profile`, `apply_component_delta`, `apply_penalty`, `recalculate_profile`, `snapshot_multiplier` |
| **realms-adapter** | [`E5CHyQY6gsxWB4cdTCSMxS3aY3J4eCVCXEe1KVTfk4Ky`](https://explorer.solana.com/address/E5CHyQY6gsxWB4cdTCSMxS3aY3J4eCVCXEe1KVTfk4Ky?cluster=devnet) | `initialize_adapter`, `bind_proposal`, `activate_council_override`, `create_voter_weight_record`, `refresh_voter_weight_record` |

All three are **native Solana programs** (not Anchor) using `solana_program::entrypoint!` with manual Borsh deserialization. Each follows the same modular layout: `state.rs`, `errors.rs`, `events.rs`, and math/helper modules.

---

## Quick Start

### Frontend

```bash
cd web && npm install && npm run dev
```

Open `http://localhost:3000`. Routes: `/` landing, `/dapp` wallet + on-chain demo, `/whitepaper` technical vision.

### Programs

```bash
cargo test                                   # unit + integration tests (52 tests)
cargo build-sbf                              # build .so programs for deployment
solana program deploy target/deploy/<name>.so --program-id target/deploy/<name>-keypair.json
```

> **Note:** `cargo build-sbf` requires temporarily commenting out `tests/integration` and `examples` from the workspace members in `Cargo.toml` (their `solana-client`/`solana-sdk` transitive deps use `edition2024` which is incompatible with the SBF toolchain's Cargo 1.84).

---

## Frontend-to-Chain Integration

The frontend calls all three programs directly using manual Borsh serialization -- zero Anchor dependency, zero additional npm packages.

**How it works:**

- `web/lib/programClient.js` contains all PDA derivation, instruction builders, account deserializers, and high-level action functions
- Every user action (register DAO, create ballot, cast vote, finalize) submits a real Solana transaction via `@solana/wallet-adapter-react`
- A transaction progress modal shows signing, sending, and confirmation stages
- On-chain proposals display Solana Explorer links and a "Refresh from chain" button to fetch live ballot tallies
- Pre-seeded demo data (BMA scenario) uses localStorage for offline preview; newly created DAOs and ballots are fully on-chain

---

## Realms Integration

Ballot Guardian is a **Voter Weight Plugin**, not a fork of SPL Governance. Realms keeps proposals and execution; we add vote weight logic.

**Integration flow:**

1. `initialize_adapter` -- register the adapter for a realm
2. `initialize_realm_config` -- configure reputation weights and bounds
3. Proposal created in Realms
4. `initialize_ballot` + `bind_proposal` -- link Realms proposal to QV ballot
5. `register_voter` + `create_voter_weight_record` -- set up voter
6. `refresh_voter_weight_record` -- compute `qv_component * rep_bps / 10_000`
7. Realms reads VWR for voting power; `cast_vote` records QV tally
8. `finalize_ballot` -- close voting

**Production steps:** Lock target SPL Governance version. Make `PluginVoterWeightRecord` layout byte-compatible with that version's VWR. Register the adapter program as the realm's `voter-weight-addin` via `set_realm_config`.

---

## Repo Layout

```
programs/
  quadratic-voting/src/   -- lib.rs, state.rs, errors.rs, events.rs, math.rs
  reputation-engine/src/  -- lib.rs, state.rs, errors.rs, events.rs, helpers.rs
  realms-adapter/src/     -- lib.rs, state.rs, errors.rs, events.rs, math.rs
tests/
  integration/            -- 52 LiteSVM integration tests (cross-program, per-program)
examples/
  client_demo.rs          -- End-to-end demo invoking all 3 programs
web/
  app/                    -- Next.js app router (landing, dapp, whitepaper)
  app/components/         -- TypewriterHeadline, MathCalculator, NetworkStatus,
                             ReputationDashboard, AuditTrail, WalletConnectButton
  lib/programClient.js    -- Borsh serialization, PDA derivation, instruction builders,
                             account deserializers, high-level action functions
  lib/demoStore.js        -- Local demo store with seeded BMA scenario + on-chain fields
  lib/anchor.js           -- Re-exports from programClient.js + program ID constants
```
