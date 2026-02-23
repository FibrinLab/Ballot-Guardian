"use client";

import { useState } from "react";

const floorSqrt = (n) => Math.floor(Math.sqrt(n));

export default function MathCalculator() {
  const [votes, setVotes] = useState(10);
  const [repBps, setRepBps] = useState(10_000);
  const [tokens, setTokens] = useState(400);

  const quadraticCost = votes * votes;
  const marginalCost = (votes + 1) * (votes + 1) - quadraticCost;
  const scaledTally = votes * repBps;
  const displayWeight = scaledTally / 10_000;

  const sqrtComponent = floorSqrt(tokens);
  const adapterWeight = Math.floor((sqrtComponent * repBps) / 10_000);

  return (
    <div className="calculator-grid">
      <div className="subpanel">
        <p className="label">VOICE CREDITS</p>
        <label htmlFor="votes-slider">Votes allocated to one option</label>
        <input
          id="votes-slider"
          type="range"
          min="1"
          max="50"
          value={votes}
          onChange={(event) => setVotes(Number(event.target.value))}
        />
        <p className="calc-line">
          <span>Votes</span> <strong>{votes}</strong>
        </p>
        <p className="calc-line">
          <span>Quadratic cost</span> <strong>{quadraticCost}</strong>
        </p>
        <p className="calc-line">
          <span>Marginal cost (+1 vote)</span> <strong>{marginalCost}</strong>
        </p>
      </div>

      <div className="subpanel">
        <p className="label">REPUTATION MULTIPLIER</p>
        <label htmlFor="rep-slider">Multiplier (basis points)</label>
        <input
          id="rep-slider"
          type="range"
          min="5000"
          max="20000"
          step="100"
          value={repBps}
          onChange={(event) => setRepBps(Number(event.target.value))}
        />
        <p className="calc-line">
          <span>Multiplier</span> <strong>{(repBps / 10_000).toFixed(2)}x</strong>
        </p>
        <p className="calc-line">
          <span>Scaled tally increment</span> <strong>{scaledTally}</strong>
        </p>
        <p className="calc-line">
          <span>Display weight</span> <strong>{displayWeight.toFixed(2)}</strong>
        </p>
      </div>

      <div className="subpanel">
        <p className="label">REALMS ADAPTER WEIGHT</p>
        <label htmlFor="token-slider">Token credits allocated (for sqrt path)</label>
        <input
          id="token-slider"
          type="range"
          min="1"
          max="10000"
          step="1"
          value={tokens}
          onChange={(event) => setTokens(Number(event.target.value))}
        />
        <p className="calc-line">
          <span>Token amount</span> <strong>{tokens}</strong>
        </p>
        <p className="calc-line">
          <span>Sqrt component</span> <strong>{sqrtComponent}</strong>
        </p>
        <p className="calc-line">
          <span>Adapter weight</span> <strong>{adapterWeight}</strong>
        </p>
      </div>
    </div>
  );
}

