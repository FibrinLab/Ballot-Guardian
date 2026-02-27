"use client";

export default function BallotResult({ result }) {
  if (!result) return null;

  const {
    outcome,
    outcomeLabel,
    yes,
    no,
    abstain,
    totalWeight,
    totalVoters,
    turnoutPct,
    quorumMet,
    marginOfVictory,
    isReputationWeighted,
    isFinalized,
    closedAt,
    ballotPDA,
    createTxSignature,
  } = result;

  const toneClass =
    outcome === "YES" ? "yes" : outcome === "NO" ? "no" : "neutral";

  const verdictTag =
    outcome === "YES"
      ? "PASSED"
      : outcome === "NO"
        ? "FAILED"
        : outcome === "TIE"
          ? "TIED"
          : "NO VOTES";

  const decidingTotal = yes + no;

  function pct(value) {
    if (totalWeight === 0) return "0";
    return Math.round((value / totalWeight) * 100);
  }

  function marginLabel(m) {
    if (m >= 60) return "Decisive";
    if (m >= 30) return "Clear";
    return "Narrow";
  }

  const quorumTone =
    quorumMet === true ? "ok" : quorumMet === false ? "warn" : "neutral";

  const closedDate = closedAt ? new Date(closedAt).toLocaleDateString() : "—";

  return (
    <div
      className={`ballot-result ballot-result--${toneClass}`}
      aria-label="Ballot result"
    >
      <div className="ballot-result__header">
        <span className="ballot-result__label">BALLOT RESULT</span>
        <div className="ballot-result__pills">
          {isReputationWeighted && (
            <span className="pill pill--onchain">Reputation-weighted</span>
          )}
          {isFinalized && (
            <span className="pill pill--onchain">Finalized on-chain</span>
          )}
          {!isReputationWeighted && (
            <span className="pill pill--muted">Local tally</span>
          )}
        </div>
      </div>

      <div className={`ballot-result__verdict ballot-result__verdict--${toneClass}`}>
        <span className="ballot-result__verdict-tag">[{verdictTag}]</span>
        <span className="ballot-result__verdict-text">{outcomeLabel}</span>
      </div>

      <div className="ballot-result__tallies">
        <div className="ballot-result__tally ballot-result__tally--yes">
          <span className="mini">YES</span>
          <strong>{yes}</strong>
          <span className="ballot-result__tally-pct">{pct(yes)}%</span>
        </div>
        <div className="ballot-result__tally ballot-result__tally--no">
          <span className="mini">NO</span>
          <strong>{no}</strong>
          <span className="ballot-result__tally-pct">{pct(no)}%</span>
        </div>
        <div className="ballot-result__tally ballot-result__tally--abstain">
          <span className="mini">ABSTAIN</span>
          <strong>{abstain}</strong>
          <span className="ballot-result__tally-pct">{pct(abstain)}%</span>
        </div>
      </div>

      <div className="ballot-result__bar">
        {yes > 0 && (
          <div
            className="ballot-result__bar-seg ballot-result__bar-seg--yes"
            style={{ width: `${decidingTotal > 0 ? (yes / decidingTotal) * 100 : 0}%` }}
          />
        )}
        {no > 0 && (
          <div
            className="ballot-result__bar-seg ballot-result__bar-seg--no"
            style={{ width: `${decidingTotal > 0 ? (no / decidingTotal) * 100 : 0}%` }}
          />
        )}
      </div>

      <div className="ballot-result__stats">
        <div className="ballot-result__stat">
          <span className="mini">TURNOUT</span>
          <strong>{turnoutPct != null ? `${turnoutPct}%` : `${totalVoters} voter${totalVoters !== 1 ? "s" : ""}`}</strong>
        </div>
        <div className="ballot-result__stat">
          <span className="mini">QUORUM</span>
          <strong data-tone={quorumTone}>
            {quorumMet === true ? "MET" : quorumMet === false ? "NOT MET" : "N/A"}
          </strong>
        </div>
        <div className="ballot-result__stat">
          <span className="mini">MARGIN</span>
          <strong>
            {marginOfVictory}%{" "}
            {decidingTotal > 0 && (
              <span className="ballot-result__margin-label">
                ({marginLabel(marginOfVictory)})
              </span>
            )}
          </strong>
        </div>
        <div className="ballot-result__stat">
          <span className="mini">WEIGHTING</span>
          <strong>{isReputationWeighted ? "Reputation QV" : "Simple count"}</strong>
        </div>
      </div>

      {ballotPDA && createTxSignature && (
        <div className="ballot-result__verify">
          <a
            href={`https://explorer.solana.com/tx/${createTxSignature}?cluster=devnet`}
            target="_blank"
            rel="noopener noreferrer"
            className="mini"
          >
            Verify on Solana Explorer
          </a>
          <span className="mini ballot-result__pda">
            Ballot: {ballotPDA.slice(0, 8)}...{ballotPDA.slice(-4)}
          </span>
        </div>
      )}

      <div className="ballot-result__stamp">
        CERTIFIED {closedDate} / BALLOT GUARDIAN v0.1
      </div>
    </div>
  );
}
