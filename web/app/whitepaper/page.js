import MathCalculator from "../components/MathCalculator";
import TypewriterHeadline from "../components/TypewriterHeadline";

const TOC_SECTIONS = [
  ["Thesis", "#thesis"],
  ["Origin", "#origin"],
  ["Architecture", "#architecture"],
  ["Modules", "#modules"],
  ["Math", "#math"],
  ["Reputation", "#signals"],
  ["Authority", "#authority"],
  ["Roadmap", "#roadmap"],
  ["Contracts", "#contracts"],
  ["Positioning", "#positioning"],
];

export default function WhitepaperPage() {
  const year = new Date().getFullYear();

  return (
    <>
      <div className="scanline" aria-hidden="true" />

      <header className="topbar">
        <div className="topbar__inner">
          <span>BALLOT GUARDIAN</span>
          <nav aria-label="Primary">
            <a href="/">Back to Home</a>
          </nav>
        </div>
      </header>

      <div className="whitepaper-layout">
        <aside className="whitepaper-toc" aria-label="Table of contents">
          <p className="label">CONTENTS</p>
          <ul>
            {TOC_SECTIONS.map(([label, href]) => (
              <li key={href}>
                <a href={href}>{label}</a>
              </li>
            ))}
          </ul>
        </aside>

        <div className="whitepaper-content">
          <header className="hero" id="top">
            <p className="kicker">WHITE PAPER v2.0 / SOLANA HACKATHON EDITION</p>
            <TypewriterHeadline
              id="hero-title"
              text="Reputation-Weighted Quadratic Voting for Realms DAOs"
            />
            <p className="lead" id="hero-subtitle">
              A governance extension for Solana organizations that need legitimacy, not just token
              balances.
            </p>
            <div className="hero__meta" role="list" aria-label="Project metadata">
              <p role="listitem">
                <span>Track</span> Realms Governance Builders / Extensions
              </p>
              <p role="listitem">
                <span>Positioning</span> Infrastructure primitive for all Realms DAOs
              </p>
              <p role="listitem">
                <span>High-impact example</span> Trade union ballots (BMA-inspired origin)
              </p>
            </div>
          </header>

          <section className="panel panel--mono" id="thesis" aria-labelledby="thesis-title">
            <div className="panel__head">
              <p className="label">EXECUTIVE SUMMARY</p>
              <span className="stamp">DRAFT / BUILDING</span>
            </div>
            <h2 id="thesis-title">
              Realms already gives governance primitives. Ballot Guardian upgrades the voting logic.
            </h2>
            <p>
              Realms does not need another dashboard. It needs a stronger voting primitive. Ballot
              Guardian adds quadratic intensity, behavior-based reputation multipliers, anti-Sybil
              guardrails, and an authority-first safety layer for institutional governance.
            </p>
            <div className="columns">
              <div className="subpanel">
                <p className="label">DEFAULT REALMS</p>
                <ul>
                  <li>
                    <code>1 token = 1 vote</code>
                  </li>
                  <li>Capital concentration risk</li>
                  <li>Weak credibility signaling</li>
                  <li>Token-buy attack exposure</li>
                </ul>
              </div>
              <div className="subpanel">
                <p className="label">BALLOT GUARDIAN</p>
                <ul>
                  <li>
                    <code>sqrt(allocated credits) * reputation</code>
                  </li>
                  <li>Intensity-based expression</li>
                  <li>On-chain credibility multipliers</li>
                  <li>Council override safety valve (optional)</li>
                </ul>
              </div>
            </div>
          </section>

          <section className="panel" id="origin" aria-labelledby="origin-title">
            <div className="panel__head">
              <p className="label">ORIGIN STORY</p>
            </div>
            <h2 id="origin-title">From broken ballots to institutional DAO infrastructure</h2>
            <p>
              This project started after seeing ballot integrity/trust problems around BMA voting.
              The framing is simple: if trade unions increasingly move toward online voting, then
              unions can be modeled as structured DAOs with stricter legitimacy requirements than a
              typical token-only governance setup.
            </p>
            <p>
              Ballot Guardian is built for that standard first, then generalized as a Realms
              primitive that any DAO can adopt.
            </p>
          </section>

          <section className="panel" id="architecture" aria-labelledby="arch-title">
            <div className="panel__head">
              <p className="label">PROGRAM ARCHITECTURE</p>
            </div>
            <h2 id="arch-title">How the three programs connect</h2>
            <pre className="arch-diagram" aria-label="Architecture diagram"><code>{`
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

  Flow:
  1. initialize_adapter + initialize_realm_config
  2. Proposal in Realms -> bind_proposal + initialize_ballot
  3. register_voter + create_voter_weight_record
  4. refresh_voter_weight_record (reads rep multiplier)
  5. Realms reads VWR for voting power
  6. cast_vote in QV for full tally
  7. finalize_ballot
`}</code></pre>
          </section>

          <section className="panel" id="modules" aria-labelledby="modules-title">
            <div className="panel__head">
              <p className="label">SYSTEM MODULES</p>
            </div>
            <h2 id="modules-title">Three on-chain modules, one governance upgrade path</h2>
            <div className="modules">
              <article className="module">
                <h3>1. Quadratic Voting Program</h3>
                <p>Tracks vote credits, enforces quadratic cost, and tallies reputation-scaled votes.</p>
                <pre aria-label="Quadratic vote flow">
                  <code>{`register_voter(...)
cast_vote(Yes|No|Abstain, additional_votes)
cost delta = new^2 - old^2
tally += votes * multiplier_bps`}</code>
                </pre>
              </article>

              <article className="module">
                <h3>2. Reputation Engine</h3>
                <p>
                  Stores deterministic reputation profiles in PDAs and computes a bounded multiplier
                  (default floor/ceiling: <code>0.5x - 2.0x</code>).
                </p>
                <pre aria-label="Reputation profile fields">
                  <code>{`participation_score
proposal_creation_score
staking_score
tenure_score
delegation_trust_score
penalties_score`}</code>
                </pre>
              </article>

              <article className="module">
                <h3>3. Realms Adapter</h3>
                <p>
                  Binds Realms proposals to Ballot Guardian ballots and publishes plugin-style voter
                  weight records for integration.
                </p>
                <pre aria-label="Adapter record formula">
                  <code>{`qv_component = qv_votes || sqrt(token_amount)
effective_weight = qv_component * rep_bps / 10_000`}</code>
                </pre>
              </article>
            </div>
          </section>

          <section className="panel" id="math" aria-labelledby="math-title">
            <div className="panel__head">
              <p className="label">INTERACTIVE MATH CHECK</p>
              <p className="mini">Client-side demo calculator (not final UI)</p>
            </div>
            <h2 id="math-title">Quadratic cost + reputation-weighted effective vote</h2>
            <MathCalculator />
          </section>

          <section className="panel" id="signals" aria-labelledby="signals-title">
            <div className="panel__head">
              <p className="label">REPUTATION MODEL</p>
            </div>
            <h2 id="signals-title">Behavior-based scoring, not token-based privilege</h2>
            <div className="columns">
              <div className="subpanel">
                <p className="label">POSITIVE SIGNALS</p>
                <ul>
                  <li>Proposal participation consistency</li>
                  <li>Successful proposal creation</li>
                  <li>Time-weighted staking / commitment</li>
                  <li>Tenure and wallet continuity</li>
                  <li>Delegation trust received</li>
                  <li>Optional credential NFTs / identity gating</li>
                </ul>
              </div>
              <div className="subpanel">
                <p className="label">NEGATIVE SIGNALS</p>
                <ul>
                  <li>Flash staking behavior</li>
                  <li>Coordinated suspicious voting patterns</li>
                  <li>Wallet cluster heuristics</li>
                  <li>Rage-quit patterns after votes</li>
                  <li>Spam proposal behavior</li>
                  <li>Manual/admin penalties for governance abuse</li>
                </ul>
              </div>
            </div>
            <p className="note">
              Hackathon MVP implementation is deterministic and rule-based. Off-chain oracle
              enrichment is optional and explicitly separated.
            </p>
          </section>

          <section className="panel" id="authority" aria-labelledby="authority-title">
            <div className="panel__head">
              <p className="label">AUTHORITY-FIRST ORGANIZATIONS</p>
            </div>
            <h2 id="authority-title">Dual-layer governance for serious organizations</h2>
            <pre aria-label="Dual-layer governance model">
              <code>{`Layer 1: Reputation-weighted quadratic voting (members)
Layer 2: Council override / safety valve (optional)

Use cases:
- trade unions
- professional bodies
- cooperatives
- regulated associations
- institutional DAOs`}</code>
            </pre>
            <p>
              This is not &quot;anti-DAO.&quot; It is a design for institutions that require member legitimacy
              and procedural safeguards at the same time.
            </p>
          </section>

          <section className="panel" id="roadmap" aria-labelledby="roadmap-title">
            <div className="panel__head">
              <p className="label">HACKATHON BUILD PLAN</p>
            </div>
            <h2 id="roadmap-title">MVP-first roadmap (what ships vs. what stretches)</h2>
            <ol className="timeline">
              <li>
                <h3>Week 1 / Core contracts</h3>
                <p>
                  Quadratic vote logic, voter credit enforcement, reputation profiles, deterministic
                  multiplier calculation, adapter binding accounts.
                </p>
              </li>
              <li>
                <h3>Week 2 / Realms integration + demo surface</h3>
                <p>
                  Proposal binding flow, plugin-style voter weight record refresh, minimal UI
                  walkthrough, example DAO configuration.
                </p>
              </li>
              <li>
                <h3>Stretch / Identity + analytics</h3>
                <p>
                  Membership NFT gating, wallet aging/cooldowns, suspicious pattern heuristics,
                  reputation dashboard.
                </p>
              </li>
            </ol>
          </section>

          <section className="panel" id="contracts" aria-labelledby="contracts-title">
            <div className="panel__head">
              <p className="label">CONTRACTS (ANCHOR)</p>
              <p className="mini">
                Implemented in <code>programs/</code>
              </p>
            </div>
            <h2 id="contracts-title">Useful MVP programs are already scaffolded and compiled</h2>
            <div className="details-grid">
              <details open>
                <summary>
                  <code>quadratic-voting</code>
                </summary>
                <ul>
                  <li>
                    <code>initialize_ballot</code>
                  </li>
                  <li>
                    <code>register_voter</code>
                  </li>
                  <li>
                    <code>update_voter_reputation_snapshot</code>
                  </li>
                  <li>
                    <code>cast_vote</code> (quadratic credit spend)
                  </li>
                  <li>
                    <code>finalize_ballot</code>
                  </li>
                </ul>
              </details>
              <details>
                <summary>
                  <code>reputation-engine</code>
                </summary>
                <ul>
                  <li>
                    <code>initialize_realm_config</code>
                  </li>
                  <li>
                    <code>set_oracle_authority</code>
                  </li>
                  <li>
                    <code>create_profile</code>
                  </li>
                  <li>
                    <code>apply_component_delta</code>
                  </li>
                  <li>
                    <code>apply_penalty</code>
                  </li>
                  <li>
                    <code>recalculate_profile</code>
                  </li>
                </ul>
              </details>
              <details>
                <summary>
                  <code>realms-adapter</code>
                </summary>
                <ul>
                  <li>
                    <code>initialize_adapter</code>
                  </li>
                  <li>
                    <code>bind_proposal</code>
                  </li>
                  <li>
                    <code>set_council_override</code>
                  </li>
                  <li>
                    <code>create_voter_weight_record</code>
                  </li>
                  <li>
                    <code>refresh_voter_weight_record</code>
                  </li>
                </ul>
              </details>
            </div>
            <p className="note">
              This adapter stores plugin-style voter weight records and proposal bindings. Final
              production compatibility should target the exact SPL Governance / Realms plugin
              interface version you plan to deploy against.
            </p>
          </section>

          <section className="panel" id="positioning" aria-labelledby="position-title">
            <div className="panel__head">
              <p className="label">POSITIONING DECISION (CURRENT DEFAULT)</p>
            </div>
            <h2 id="position-title">
              Primary message: infrastructure primitive for all Realms DAOs
            </h2>
            <p>
              The site and docs are currently written with <strong>Option C</strong> as the headline
              strategy, while using trade unions as the high-impact, concrete motivating example.
            </p>
            <p>
              If you want, this can be re-skinned into an explicitly union-first pitch in one pass
              without changing the underlying contracts.
            </p>
          </section>
        </div>
      </div>

      <footer className="footer">
        <p>
          Ballot Guardian / Solana Edition / <span>{year}</span> / built for Realms-track
          governance experimentation.
        </p>
      </footer>
    </>
  );
}
