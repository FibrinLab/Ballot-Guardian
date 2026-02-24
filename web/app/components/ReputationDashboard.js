"use client";

import { useState } from "react";

const BASE_BPS = 10_000;
const POINTS_PER_BONUS_BPS = 100;
const PENALTY_UNIT_BPS = 500;
const MAX_BONUS_BPS = 5_000;
const MAX_PENALTY_BPS = 5_000;
const MIN_MULTIPLIER_BPS = 5_000;
const MAX_MULTIPLIER_BPS = 20_000;
const WEIGHTS = [30, 20, 20, 15, 15];

const COMPONENTS = [
  { key: "participation", label: "Participation", weight: WEIGHTS[0] },
  { key: "proposalCreation", label: "Proposals", weight: WEIGHTS[1] },
  { key: "staking", label: "Staking", weight: WEIGHTS[2] },
  { key: "tenure", label: "Tenure", weight: WEIGHTS[3] },
  { key: "delegation", label: "Delegation", weight: WEIGHTS[4] },
];

function computeMultiplier(scores, penalties) {
  let weightedPoints = 0;
  for (let i = 0; i < COMPONENTS.length; i++) {
    weightedPoints += scores[COMPONENTS[i].key] * COMPONENTS[i].weight;
  }

  const bonusBps = Math.min(
    Math.floor(weightedPoints / POINTS_PER_BONUS_BPS),
    MAX_BONUS_BPS,
  );

  const penaltyBps = Math.min(penalties * PENALTY_UNIT_BPS, MAX_PENALTY_BPS);

  const multiplierBps = Math.min(
    MAX_MULTIPLIER_BPS,
    Math.max(MIN_MULTIPLIER_BPS, BASE_BPS + bonusBps - penaltyBps),
  );

  return { weightedPoints, bonusBps, penaltyBps, multiplierBps };
}

export default function ReputationDashboard() {
  const [scores, setScores] = useState({
    participation: 72,
    proposalCreation: 45,
    staking: 88,
    tenure: 60,
    delegation: 33,
  });
  const [penalties, setPenalties] = useState(1);

  const { weightedPoints, bonusBps, penaltyBps, multiplierBps } =
    computeMultiplier(scores, penalties);
  const multiplier = (multiplierBps / 10_000).toFixed(2);

  function handleScore(key, value) {
    setScores((prev) => ({ ...prev, [key]: Number(value) }));
  }

  return (
    <div className="rep-dashboard">
      <div className="subpanel">
        <p className="label">REPUTATION MULTIPLIER</p>
        <p className="rep-multiplier">{multiplier}x</p>
        <p className="mini">
          base {(BASE_BPS / 10_000).toFixed(1)}x + bonus{" "}
          {(bonusBps / 10_000).toFixed(2)}x - penalty{" "}
          {(penaltyBps / 10_000).toFixed(2)}x
        </p>
      </div>

      <div className="subpanel">
        <p className="label">COMPONENT SCORES</p>
        <p className="mini" style={{ marginBottom: 8 }}>
          Drag sliders to adjust scores. Weighted total: {weightedPoints} pts
        </p>
        <div className="rep-bar-group">
          {COMPONENTS.map(({ key, label, weight }) => {
            const value = scores[key];
            const pct = Math.min((value / 100) * 100, 100);
            return (
              <div key={key} className="rep-bar-row">
                <span className="rep-bar-label">
                  {label} <span className="rep-weight">w{weight}</span>
                </span>
                <div className="rep-slider-track">
                  <div className="rep-bar-track">
                    <div className="rep-bar-fill" style={{ width: `${pct}%` }} />
                  </div>
                  <input
                    type="range"
                    min={0}
                    max={100}
                    value={value}
                    onChange={(e) => handleScore(key, e.target.value)}
                    className="rep-range-input"
                    aria-label={`${label} score`}
                  />
                </div>
                <span className="rep-bar-value">{value}</span>
              </div>
            );
          })}
        </div>
      </div>

      <div className="subpanel">
        <p className="label">PENALTIES</p>
        <div className="rep-bar-row">
          <span className="rep-bar-label">Penalty pts</span>
          <div className="rep-slider-track">
            <input
              type="range"
              min={0}
              max={10}
              value={penalties}
              onChange={(e) => setPenalties(Number(e.target.value))}
              className="rep-range-input"
              aria-label="Penalty points"
            />
          </div>
          <span className="rep-bar-value">{penalties}</span>
        </div>
        <p className="mini" style={{ marginTop: 6 }}>
          Each point reduces multiplier by {PENALTY_UNIT_BPS} bps (capped at{" "}
          {MAX_PENALTY_BPS} bps).
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
