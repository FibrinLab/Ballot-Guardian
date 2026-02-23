import Link from "next/link";

export default function LandingPage() {
  return (
    <>
      <div className="scanline" aria-hidden="true" />

      <header className="topbar">
        <div className="topbar__inner">
          <span>BALLOT GUARDIAN / SOLANA</span>
          <nav aria-label="Primary">
            <Link href="/dapp">Dapp</Link>
            <Link href="/whitepaper">White Paper</Link>
          </nav>
        </div>
      </header>

      <main className="page landing">
        <section className="hero landing-hero">
          <p className="kicker">TRUSTED ONCHAIN VOTING FOR UNIONS + REALMS DAOS</p>
          <h1>Register a union. Create a ballot. Vote with your wallet.</h1>
          <p className="lead">
            Ballot Guardian is a Solana governance application and Realms extension concept built
            for legitimate digital ballots in serious organizations.
          </p>
          <div className="button-row">
            <Link className="button" href="/dapp">
              Open Dapp
            </Link>
            <Link className="button button--ghost" href="/whitepaper">
              Read White Paper
            </Link>
          </div>
        </section>

        <section className="panel">
          <div className="panel__head">
            <p className="label">HOW IT WORKS (MVP)</p>
          </div>
          <div className="landing-grid">
            <article className="subpanel">
              <h2>1. Connect Wallet</h2>
              <p>
                Use Phantom / Solflare / Backpack to authenticate as a member or organizer.
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
                Cast a wallet-backed vote. Next step is wiring this UI directly to the Anchor
                contracts already in the repo.
              </p>
            </article>
          </div>
        </section>

        <section className="panel">
          <div className="panel__head">
            <p className="label">WHY THIS MATTERS</p>
          </div>
          <div className="columns">
            <div className="subpanel">
              <p className="label">UNION ORIGIN</p>
              <p>
                Inspired by ballot integrity concerns and the need for online voting systems that
                can be independently verified and audited.
              </p>
            </div>
            <div className="subpanel">
              <p className="label">REALMS TRACK FIT</p>
              <p>
                The same primitives can plug into Realms governance with quadratic voting and
                reputation-weighted extensions for institutional DAOs.
              </p>
            </div>
          </div>
        </section>
      </main>
    </>
  );
}

