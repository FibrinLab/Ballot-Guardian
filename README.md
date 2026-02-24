# Ballot Guardian

Reputation-weighted quadratic voting for Solana Realms DAOs.

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

## Programs

| Program | Instructions | Purpose |
|---------|-------------|---------|
| **quadratic-voting** | `initialize_ballot`, `register_voter`, `update_voter_reputation_snapshot`, `cast_vote`, `finalize_ballot` | Quadratic credit spend, reputation-scaled tallies |
| **reputation-engine** | `initialize_realm_config`, `set_oracle_authority`, `create_profile`, `apply_component_delta`, `apply_penalty`, `recalculate_profile`, `snapshot_multiplier` | Behavior-based scoring, bounded multiplier computation |
| **realms-adapter** | `initialize_adapter`, `bind_proposal`, `set_council_override`, `create_voter_weight_record`, `refresh_voter_weight_record` | Binds Realms proposals to QV ballots, publishes voter weight records |

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
cargo check
cargo test
```

For full Anchor workflows (`anchor build`, IDL generation), install `anchor-cli`.

---

## Demo Flow

1. **Connect wallet** -- open `/dapp`, connect Phantom (Solana devnet)
2. **Register a union/DAO** -- fill in name + description, sign with wallet
3. **Create a ballot** -- add a proposal question and voting window
4. **Cast a vote** -- select Yes / No / Abstain, sign with wallet
5. **View audit trail** -- scroll down to see all wallet-signed actions with truncated signatures
6. **View reputation** -- simulated reputation dashboard shows multiplier computation

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
web/
  app/                    -- Next.js app router (landing, dapp, whitepaper)
  app/components/         -- TypewriterHeadline, MathCalculator, NetworkStatus,
                             ReputationDashboard, AuditTrail, WalletConnectButton
```
