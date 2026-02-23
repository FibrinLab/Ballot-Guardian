# Contract Overview

This document summarizes the MVP Anchor contracts in `programs/`.

## `quadratic-voting`

### Key Accounts

- `QuadraticBallot`
  - proposal/realm identifiers
  - authority
  - voting window
  - reputation bounds
  - scaled tallies (`yes/no/abstain`)
- `VoterAllocation`
  - voter pubkey
  - credit budget / spent
  - vote allocations by choice
  - reputation multiplier snapshot

### Key Behavior

- quadratic cost is enforced incrementally (`new^2 - old^2`)
- votes are additive per choice
- tallies are stored in basis-point-scaled precision (`votes * multiplier_bps`)

## `reputation-engine`

### Key Accounts

- `RealmReputationConfig`
  - scoring weights
  - min/base/max multiplier bounds
  - penalty and bonus calibration
  - admin/oracle authority
- `ReputationProfile`
  - component scores
  - penalties score
  - computed multiplier

### Key Behavior

- component updates use signed deltas (with saturating floor at zero)
- penalties are explicit and bounded
- multiplier is recomputed deterministically and clamped

## `realms-adapter`

### Key Accounts

- `AdapterConfig`
  - realm and program references
  - admin + council override authority
  - reputation bounds
- `ProposalBinding`
  - Realms proposal pubkey
  - quadratic ballot pubkey
  - governing token mint
  - council override flags
- `PluginVoterWeightRecord`
  - voter weight and metadata
  - action target / expiry slot
  - QV and reputation inputs used for computation

### Key Behavior

- stores proposal bindings independently of vote allocations
- computes effective voter weight from QV + reputation
- exposes a clean integration seam for exact plugin compatibility work

