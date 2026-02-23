# Ballot Guardian Hackathon Plan (Detailed)

## Recommended Positioning

Primary: **Infrastructure primitive for all Realms DAOs (Option C)**

Why:

- strongest fit for Realms track judging criteria (extensibility + reusability)
- avoids narrowing the pitch too early
- still allows a memorable, high-stakes trade union demo narrative

Use trade unions / BMA-style ballot reliability issues as the motivating example, not the only market.

## Product Thesis

Realms already provides governance rails. Ballot Guardian improves the voting primitive by adding:

- quadratic allocation (intensity expression)
- on-chain reputation multipliers (credibility weighting)
- anti-Sybil guardrails (time/identity/cooldown patterns)
- optional council override safety valve (institutional compatibility)

## Hackathon MVP Scope (What Must Ship)

### 1. On-chain contracts (MVP)

- `quadratic-voting` program
  - ballot init
  - voter registration / credit budget
  - cast vote with quadratic cost enforcement
  - finalize ballot
- `reputation-engine` program
  - realm config
  - profile creation
  - component updates and penalties
  - multiplier recomputation
- `realms-adapter` program
  - adapter config
  - proposal binding
  - voter weight record create/refresh
  - council override state flag

### 2. Demonstration interface (MVP)

- one-page, black/white typewriter presentation page
- architecture + math explanation
- interactive calculator for QV cost and reputation-weighted output
- explicit union/institutional use case framing

### 3. Technical proof points (MVP)

- `cargo check` and `cargo test` passing
- PDA/account schemas documented
- explicit Realms plugin integration path documented

## Post-MVP / Stretch Goals

- exact SPL Governance add-in record compatibility for a target Realms version
- wallet aging / cooldown enforcement on-chain
- credential/NFT gating integration
- local validator demo that simulates proposal -> voter weight refresh -> vote cast
- reputation dashboard + event indexing

## Demo Narrative (Recommended)

1. Problem: token-only voting is vulnerable to capital concentration and legitimacy concerns.
2. Real-world motivation: union/professional body ballots require trust and auditability.
3. Realms fit: do not replace governance framework; extend it.
4. Show math: quadratic cost and reputation multiplier.
5. Show contracts: QV + reputation + adapter.
6. Show institutional safety valve: council override option.
7. Close with reusable plugin packaging vision.

## Execution Plan (2-Week Sprint)

### Week 1

- finish contract correctness and constraints
- establish account schemas and events
- add unit tests for math and scoring
- write Realms adapter integration mapping doc

### Week 2

- build demo UI with proposal/voter examples
- connect demo to a mock/local flow or simulated transaction narratives
- produce pitch deck / submission write-up
- harden docs and record final architecture diagram

## Risk Register

### Risk: Realms plugin compatibility mismatch

Mitigation:

- lock a specific SPL Governance / Realms plugin interface version early
- treat current adapter as a stable internal abstraction until the interface is finalized

### Risk: Overbuilding anti-Sybil heuristics

Mitigation:

- ship deterministic rule-based penalties and cooldown hooks first
- make heuristic/oracle layer optional in MVP

### Risk: UI complexity drift

Mitigation:

- keep site ultra-minimal and typewriter-style
- prioritize technical clarity over dashboard polish

## Open Decisions (Needs Your Answers)

1. Keep primary positioning as **Option C** with unions as a flagship example?
2. Which Realms / SPL Governance version do you want to target first for plugin compatibility?
3. Do you want the next implementation step to be:
   - exact plugin account compatibility
   - local demo transactions
   - union-specific UX flow (member onboarding / identity gate)

