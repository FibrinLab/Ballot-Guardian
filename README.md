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

**Judges**: start at `/dapp` -- it opens with a pre-populated BMA Junior Doctors scenario (industrial action ballot with sample votes).

1. **Connect wallet** -- open `/dapp`, connect Phantom (Solana devnet)
2. **Explore pre-loaded ballots** -- two proposals with live vote tallies
3. **Register a new union/DAO** -- fill in name + description, sign with wallet
4. **Create a ballot** -- add a proposal question and voting window
5. **Cast a vote** -- select Yes / No / Abstain, sign with wallet
6. **View audit trail** -- scroll down to see all wallet-signed actions with truncated signatures
7. **View reputation** -- reputation dashboard shows multiplier computation from five scoring components

---

## Deployed Programs (Solana Devnet)

| Program | Address | Instructions |
|---------|---------|-------------|
| **quadratic-voting** | [`346RNEQcBBff4skHhQiRCPe9cDeWpaPTsT2TpQUFYomp`](https://explorer.solana.com/address/346RNEQcBBff4skHhQiRCPe9cDeWpaPTsT2TpQUFYomp?cluster=devnet) | `initialize_ballot`, `register_voter`, `update_voter_reputation_snapshot`, `cast_vote`, `finalize_ballot` |
| **reputation-engine** | [`8JpbKjoR4c7n2HqS51WjyjJrLwvVgGGsKN4o2boohdEA`](https://explorer.solana.com/address/8JpbKjoR4c7n2HqS51WjyjJrLwvVgGGsKN4o2boohdEA?cluster=devnet) | `initialize_realm_config`, `set_oracle_authority`, `create_profile`, `apply_component_delta`, `apply_penalty`, `recalculate_profile`, `snapshot_multiplier` |
| **realms-adapter** | [`E5CHyQY6gsxWB4cdTCSMxS3aY3J4eCVCXEe1KVTfk4Ky`](https://explorer.solana.com/address/E5CHyQY6gsxWB4cdTCSMxS3aY3J4eCVCXEe1KVTfk4Ky?cluster=devnet) | `initialize_adapter`, `bind_proposal`, `set_council_override`, `create_voter_weight_record`, `refresh_voter_weight_record` |

All three programs follow the same modular layout: `state.rs`, `errors.rs`, `events.rs`, and math/helper modules.

---

## Quick Start

### Frontend

```bash
cd web && npm install && npm run dev
```

Open `http://localhost:3000`. Routes: `/` landing, `/dapp` wallet + union/ballot demo, `/whitepaper` technical vision.

### Contracts

```bash
cargo test                                   # unit + integration tests (52 tests)
cargo build-sbf                              # build .so programs for deployment
solana program deploy target/deploy/<name>.so --program-id target/deploy/<name>-keypair.json
```

> **Note:** `cargo build-sbf` requires temporarily commenting out `tests/integration` and `examples` from the workspace members in `Cargo.toml` (their `solana-client`/`solana-sdk` transitive deps use `edition2024` which is incompatible with the SBF toolchain's Cargo 1.84).

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
  lib/demoStore.js        -- Local demo store with seeded BMA scenario
  lib/anchor.js           -- Solana client plumbing with deployed program IDs
  lib/idl/                -- Drop IDL JSONs here after `anchor build`
```
