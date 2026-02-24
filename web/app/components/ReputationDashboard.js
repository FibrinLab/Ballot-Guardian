"use client";

const SIMULATED_PROFILE = {
  participation: 72,
  proposalCreation: 45,
  staking: 88,
  tenure: 60,
  delegation: 33,
  penalties: 1,
  multiplierBps: 12_500,
  baseBps: 10_000,
  bonusBps: 3_000,
  penaltyBps: 500,
};

const COMPONENTS = [
  { key: "participation", label: "Participation", max: 100 },
  { key: "proposalCreation", label: "Proposals", max: 100 },
  { key: "staking", label: "Staking", max: 100 },
  { key: "tenure", label: "Tenure", max: 100 },
  { key: "delegation", label: "Delegation", max: 100 },
];

export default function ReputationDashboard() {
  const p = SIMULATED_PROFILE;
  const multiplier = (p.multiplierBps / 10_000).toFixed(2);

  return (
    <div className="rep-dashboard">
      <div className="subpanel">
        <p className="label">REPUTATION MULTIPLIER</p>
        <p className="rep-multiplier">{multiplier}x</p>
        <p className="mini">
          Simulated profile / base {(p.baseBps / 10_000).toFixed(1)}x + bonus{" "}
          {(p.bonusBps / 10_000).toFixed(2)}x - penalty{" "}
          {(p.penaltyBps / 10_000).toFixed(2)}x
        </p>
      </div>

      <div className="subpanel">
        <p className="label">COMPONENT SCORES</p>
        <div className="rep-bar-group">
          {COMPONENTS.map(({ key, label, max }) => {
            const value = p[key];
            const pct = Math.min((value / max) * 100, 100);
            return (
              <div key={key} className="rep-bar-row">
                <span className="rep-bar-label">{label}</span>
                <div className="rep-bar-track">
                  <div className="rep-bar-fill" style={{ width: `${pct}%` }} />
                </div>
                <span className="rep-bar-value">{value}</span>
              </div>
            );
          })}
        </div>
      </div>

      <div className="subpanel">
        <p className="label">PENALTIES</p>
        <p className="mono-value">{p.penalties} penalty point(s) applied</p>
        <p className="mini">
          Each penalty point reduces multiplier by{" "}
          <code>penalty_unit_bps</code> (capped at <code>max_penalty_bps</code>).
        </p>
      </div>

      <div className="subpanel rep-formula">
        <p className="label">FORMULA</p>
        <pre>
          <code>{`weighted_points = SUM(score_i * weight_i)
bonus_bps = min(weighted_points / points_per_bonus_bps, max_bonus_bps)
penalty_bps = min(penalties * penalty_unit_bps, max_penalty_bps)
multiplier = clamp(base + bonus - penalty, min, max)`}</code>
        </pre>
      </div>
    </div>
  );
}
