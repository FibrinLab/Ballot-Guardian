# Realms Integration Notes

Ballot Guardian is designed as a **Realms governance extension** (plugin/add-in model) rather than a
fork of SPL Governance.

## Core Principle

Do not rebuild governance framework primitives (proposals, execution, DAO settings) that Realms
already provides. Instead, replace/augment vote weight logic and add auditable reputation + QV state.

## Current On-Chain Components in This Repo

### 1. `quadratic-voting`

Owns:

- proposal-level ballot PDA
- voter credit budget / vote allocation PDA
- quadratic credit spending enforcement
- reputation-scaled vote tallies

### 2. `reputation-engine`

Owns:

- realm reputation configuration PDA
- member reputation profile PDAs
- deterministic multiplier computation (bounded min/max)

### 3. `realms-adapter`

Owns:

- adapter config per realm
- proposal binding between Realms proposal and Ballot Guardian ballot
- plugin-style voter weight record PDA
- optional council override signaling state

## Integration Model (Practical MVP)

The adapter currently stores a plugin-facing voter weight record abstraction:

- proposal binding (`ProposalBinding`)
- voter weight record (`PluginVoterWeightRecord`)

Weight is computed as:

```text
qv_component = qv_votes_allocated OR floor(sqrt(token_amount_allocated))
effective_weight = qv_component * reputation_multiplier_bps / 10_000
```

This is intentionally simple and deterministic for hackathon delivery.

## Intended Realms Flow (Target State)

1. DAO creates proposal in Realms.
2. Ballot Guardian adapter binds Realms proposal pubkey -> quadratic ballot PDA.
3. Reputation engine snapshots multiplier for voter (or provides current multiplier).
4. Adapter refreshes voter weight record for the proposal.
5. Realms governance program reads voter weight record during vote action.
6. Quadratic voting program stores allocation/cost/tally details for auditability.

## Why the Adapter Exists

Realms/SPL Governance plugin compatibility is version-sensitive. The adapter lets you:

- keep Ballot Guardian business logic stable
- map to the exact add-in record layout for a chosen Realms version later
- avoid coupling QV/reputation contracts to a moving plugin interface too early

## What Needs to Be Finalized for Production Compatibility

### 1. Exact SPL Governance add-in record layouts

- `VoterWeightRecord`
- `MaxVoterWeightRecord` (if needed by target governance flow)

### 2. Action targeting semantics

- proposal-scoped vs governance-scoped weight records
- expiry slot behavior expected by the target plugin interface

### 3. CPI and authorization rules

- which program is allowed to refresh weight records
- whether Realms governance program or an external relayer invokes refresh logic

## Suggested Next Integration Step

Lock a specific Realms/SPL Governance version, then update `realms-adapter` account layouts to be
byte-compatible with the corresponding add-in interface while preserving the current proposal
binding and weight computation logic.

