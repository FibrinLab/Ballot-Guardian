"use client";

import { useState } from "react";

function compactAddress(addr) {
  if (!addr || addr.length < 10) return addr || "n/a";
  return `${addr.slice(0, 4)}...${addr.slice(-4)}`;
}

function compactSig(hex) {
  if (!hex || hex.length < 16) return hex || "";
  return `${hex.slice(0, 8)}...${hex.slice(-8)}`;
}

function formatTime(iso) {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleString();
}

function badgeLabel(kind) {
  if (kind === "register_dao") return "REGISTER";
  if (kind === "create_ballot") return "BALLOT";
  if (kind === "cast_vote") return "VOTE";
  return kind?.toUpperCase() || "ACTION";
}

function formatProofJson(proof) {
  if (!proof) return null;
  try {
    const payload =
      typeof proof.message === "string"
        ? JSON.parse(proof.message)
        : proof.message;
    return JSON.stringify(
      {
        message: payload,
        signatureHex: proof.signatureHex,
        wallet: proof.wallet,
        signedAt: proof.signedAt,
      },
      null,
      2,
    );
  } catch {
    return JSON.stringify(proof, null, 2);
  }
}

function CopyButton({ text }) {
  const [copied, setCopied] = useState(false);

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      /* clipboard not available */
    }
  }

  return (
    <button
      type="button"
      className="button button--ghost audit-copy-btn"
      onClick={handleCopy}
    >
      {copied ? "Copied" : "Copy"}
    </button>
  );
}

export default function AuditTrail({ store, daoId }) {
  if (!store || !daoId) {
    return <div className="empty-state">Select a DAO to view its audit trail.</div>;
  }

  const dao = store.daos.find((d) => d.id === daoId);
  if (!dao) {
    return <div className="empty-state">DAO not found.</div>;
  }

  const entries = [];

  if (dao.proof) {
    entries.push({
      kind: "register_dao",
      wallet: dao.proof.wallet,
      signatureHex: dao.proof.signatureHex,
      signedAt: dao.proof.signedAt,
      proof: dao.proof,
    });
  }

  for (const proposal of dao.proposals || []) {
    if (proposal.proof) {
      entries.push({
        kind: "create_ballot",
        wallet: proposal.proof.wallet,
        signatureHex: proposal.proof.signatureHex,
        signedAt: proposal.proof.signedAt,
        proof: proposal.proof,
      });
    }
    for (const vote of proposal.votes || []) {
      if (vote.proof) {
        entries.push({
          kind: "cast_vote",
          wallet: vote.proof.wallet,
          signatureHex: vote.proof.signatureHex,
          signedAt: vote.proof.signedAt,
          proof: vote.proof,
        });
      }
    }
  }

  entries.sort((a, b) => new Date(b.signedAt) - new Date(a.signedAt));

  if (entries.length === 0) {
    return <div className="empty-state">No signed actions yet for this DAO.</div>;
  }

  return (
    <div className="audit-trail" role="list" aria-label="Audit trail">
      {entries.map((entry, i) => {
        const proofJson = formatProofJson(entry.proof);

        return (
          <details key={i} className="audit-entry-details" role="listitem">
            <summary className="audit-entry">
              <span className="audit-badge">{badgeLabel(entry.kind)}</span>
              <span className="audit-detail">
                {compactAddress(entry.wallet)} / {formatTime(entry.signedAt)}
              </span>
              <code className="audit-sig">{compactSig(entry.signatureHex)}</code>
            </summary>
            {proofJson ? (
              <div className="audit-expanded">
                <div className="audit-expanded__head">
                  <span className="mini">Full signed proof</span>
                  <CopyButton text={proofJson} />
                </div>
                <pre className="audit-expanded__json">
                  <code>{proofJson}</code>
                </pre>
              </div>
            ) : (
              <div className="audit-expanded">
                <p className="mini">
                  No cryptographic proof attached (seeded demo data).
                </p>
              </div>
            )}
          </details>
        );
      })}
    </div>
  );
}
