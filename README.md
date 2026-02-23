# Ballot Guardian

Ballot Guardian is a Solana-first governance extension for Realms that introduces:

- quadratic voting
- reputation-weighted vote multipliers
- anti-sybil guardrails
- authority-first safety layers

This repository is structured as a hackathon-ready build scaffold with useful on-chain MVP contracts:

- `web/`: ultrasimplistic black/white typewriter-style product/vision site
- `programs/`: Anchor-compatible Solana programs for quadratic voting, reputation, and a Realms adapter
- `docs/`: integration notes and architecture guidance

## Status

- Anchor contracts implemented and compiling (`cargo check`, `cargo test`)
- Static manifesto/demo site implemented
- Realms integration documented as plugin-facing adapter flow (version-specific compatibility still to finalize)

## Tooling Notes

- `anchor-cli` was not installed in the original build session, but the contracts were still authored in Anchor format and verified with `cargo`.
- Rust (`cargo`) is available.

## Quick Start

### Web (static)

Open `web/index.html` in a browser, or serve the folder:

```bash
cd web
python3 -m http.server 8080
```

### Contracts (Rust/Anchor-compatible)

```bash
cargo check
cargo test
```

If you want full Anchor workflows (`anchor build`, `anchor test`, IDL generation), install `anchor-cli`.

## Programs Overview

### `quadratic-voting`

- Initializes proposal-specific quadratic ballot PDAs
- Registers voters with credit budgets and reputation multiplier snapshots
- Enforces quadratic cost when casting additive votes
- Maintains reputation-scaled tallies (`u128` basis-point scaled)

### `reputation-engine`

- Stores realm-level scoring config
- Maintains per-member reputation profiles in PDAs
- Supports component deltas + explicit penalties
- Computes bounded multiplier (`base +/- score/penalties`, clamped to min/max)

### `realms-adapter`

- Binds Realms proposal pubkeys to Ballot Guardian ballots
- Stores plugin-style voter weight records
- Computes effective weight from QV component + reputation multiplier
- Includes optional council-override signal state

## Positioning (Current Default)

Primary pitch is `C) Infrastructure primitive for all Realms DAOs`, with trade unions as a high-impact example and origin story.

## Next Steps

1. Lock target Realms / SPL Governance plugin interface version.
2. Add integration tests with real SPL Governance add-in record layouts.
3. Build a minimal vote-casting demo UI against local validator.
