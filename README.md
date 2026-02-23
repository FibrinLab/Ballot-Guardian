# Ballot Guardian

Ballot Guardian is a Solana-first governance extension concept for Realms:

- quadratic voting
- reputation-weighted vote multipliers
- anti-sybil guardrails
- authority-first safety layers

This repository is a greenfield hackathon scaffold with:

- `web/`: ultrasimplistic black/white typewriter-style product/vision site
- `programs/`: Anchor-compatible Solana programs for quadratic voting, reputation, and a Realms adapter scaffold
- `docs/`: integration notes and architecture guidance

## Status

Initial scaffold. Core contract logic and Realms adapter behaviors are implemented incrementally in follow-up commits.

## Tooling Notes

- `anchor-cli` is not installed in this environment, so contracts are authored in Anchor format but may not be compiled locally until you install it.
- Rust (`cargo`) is available.

