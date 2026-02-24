import Link from "next/link";
import TypewriterHeadline from "./components/TypewriterHeadline";
import NetworkStatus from "./components/NetworkStatus";

export default function LandingPage() {
  const year = new Date().getFullYear();

  return (
    <>
      <div className="scanline" aria-hidden="true" />

      <header className="topbar">
        <div className="topbar__inner">
          <Link href="/" className="topbar__brand">BALLOT GUARDIAN / SOLANA</Link>
          <nav aria-label="Primary">
            <Link href="/dapp">Dapp</Link>
            <Link href="/whitepaper">White Paper</Link>
          </nav>
        </div>
      </header>

      <main className="page landing">
        <section className="hero landing-hero">
          <p className="kicker">SOLANA REALMS TRACK</p>
          <TypewriterHeadline
            id="hero-title"
            text="My union votes on strike action by post. I built this to fix that."
          />
          <p className="lead">
            Ballot Guardian is a Solana governance application and Realms extension
            built by an NHS junior doctor to bring verifiable, wallet-signed voting
            to the British Medical Association and organizations like it.
          </p>
          <div className="button-row">
            <Link className="button" href="/dapp">
              Open Dapp
            </Link>
            <Link className="button button--ghost" href="/whitepaper">
              Read WhitePaper
            </Link>
          </div>
        </section>

        <section className="panel">
          <div className="panel__head">
            <p className="label">HOW IT WORKS</p>
            <NetworkStatus />
          </div>
          <div className="landing-grid">
            <article className="subpanel">
              <h2>1. Connect Wallet</h2>
              <p>
                Use Phantom wallet to authenticate as a member or organizer.
              </p>
            </article>
            <article className="subpanel">
              <h2>2. Register Union (DAO)</h2>
              <p>
                Create a union workspace, then add proposals/ballots for members to vote on.
              </p>
            </article>
            <article className="subpanel">
              <h2>3. Vote + Audit</h2>
              <p>
                Cast a wallet-backed vote. Every action is signed and recorded in the audit trail.
              </p>
            </article>
          </div>
        </section>

        <section className="panel">
          <div className="panel__head">
            <p className="label">PAPER BALLOTS VS BALLOT GUARDIAN</p>
          </div>
          <div className="comparison-table" role="table" aria-label="Feature comparison">
            <div className="comparison-table__header" role="row">
              <span role="columnheader">Feature</span>
              <span role="columnheader">Paper Ballot</span>
              <span role="columnheader">Ballot Guardian</span>
            </div>
            <div className="comparison-table__row" role="row">
              <span role="cell">Independently auditable</span>
              <span role="cell" className="comparison-x" aria-label="No">&#x2717;</span>
              <span role="cell" className="comparison-check" aria-label="Yes">&#x2713;</span>
            </div>
            <div className="comparison-table__row" role="row">
              <span role="cell">Wallet-verified identity</span>
              <span role="cell" className="comparison-x" aria-label="No">&#x2717;</span>
              <span role="cell" className="comparison-check" aria-label="Yes">&#x2713;</span>
            </div>
            <div className="comparison-table__row" role="row">
              <span role="cell">Quadratic cost fairness</span>
              <span role="cell" className="comparison-x" aria-label="No">&#x2717;</span>
              <span role="cell" className="comparison-check" aria-label="Yes">&#x2713;</span>
            </div>
            <div className="comparison-table__row" role="row">
              <span role="cell">Reputation-weighted</span>
              <span role="cell" className="comparison-x" aria-label="No">&#x2717;</span>
              <span role="cell" className="comparison-check" aria-label="Yes">&#x2713;</span>
            </div>
            <div className="comparison-table__row" role="row">
              <span role="cell">Real-time tallying</span>
              <span role="cell" className="comparison-x" aria-label="No">&#x2717;</span>
              <span role="cell" className="comparison-check" aria-label="Yes">&#x2713;</span>
            </div>
            <div className="comparison-table__row" role="row">
              <span role="cell">Tamper-evident</span>
              <span role="cell" className="comparison-x" aria-label="No">&#x2717;</span>
              <span role="cell" className="comparison-check" aria-label="Yes">&#x2713;</span>
            </div>
          </div>
        </section>

        <section className="panel">
          <div className="panel__head">
            <p className="label">WHY THIS MATTERS</p>
          </div>
          <div className="columns">
            <div className="subpanel">
              <p className="label">THE PROBLEM</p>
              <p>
                The BMA represents over 190,000 doctors. Strike ballots are
                conducted by post and email -- opaque processes with no independent
                audit trail, no way for members to verify their vote was counted,
                and a trust deficit that undermines collective action.
              </p>
            </div>
            <div className="subpanel">
              <p className="label">THE SOLUTION</p>
              <p>
                Wallet-signed votes on Solana give every member a cryptographic
                receipt. Quadratic cost prevents whales from dominating. Reputation
                multipliers reward consistent participation. The entire system is
                Realms-compatible, so existing DAOs can adopt it as a voter weight
                plugin.
              </p>
            </div>
          </div>
        </section>

        <section className="panel">
          <div className="panel__head">
            <p className="label">EXPLORE ARCHITECTURE</p>
          </div>
          <h2>Three programs, one governance upgrade path</h2>
          <p>
            Quadratic Voting, Reputation Engine, and Realms Adapter work together as a Voter Weight
            Plugin for Realms. Read the full technical breakdown in the whitepaper.
          </p>
          <div className="button-row">
            <Link className="button button--ghost" href="/whitepaper#modules">
              View System Modules
            </Link>
            <Link className="button button--ghost" href="/whitepaper#architecture">
              View Architecture Diagram
            </Link>
          </div>
        </section>
      </main>

      <footer className="footer">
        <p>
          Ballot Guardian / Solana Edition / <span>{year}</span> / built for Realms-track
          governance experimentation.
        </p>
      </footer>
    </>
  );
}
