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
  if (kind === "close_ballot") return "CLOSE";
  if (kind === "delete_dao") return "DELETE";
  return kind?.toUpperCase() || "ACTION";
}

function explorerTxUrl(sig) {
  return `https://explorer.solana.com/tx/${sig}?cluster=devnet`;
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

  // DAO registration
  if (dao.proof || dao.initTxSignature) {
    entries.push({
      kind: "register_dao",
      wallet: dao.proof?.wallet || dao.createdBy,
      signatureHex: dao.proof?.signatureHex,
      signedAt: dao.proof?.signedAt || dao.createdAt,
      proof: dao.proof,
      txSignature: dao.initTxSignature,
      onChainDetails: dao.realmPubkey
        ? `Realm: ${compactAddress(dao.realmPubkey)}`
        : null,
    });
  }

  for (const proposal of dao.proposals || []) {
    // Ballot creation
    if (proposal.proof || proposal.createTxSignature) {
      entries.push({
        kind: "create_ballot",
        wallet: proposal.proof?.wallet || proposal.createdBy,
        signatureHex: proposal.proof?.signatureHex,
        signedAt: proposal.proof?.signedAt || proposal.createdAt,
        proof: proposal.proof,
        txSignature: proposal.createTxSignature,
        onChainDetails: proposal.ballotPDA
          ? `Ballot PDA: ${compactAddress(proposal.ballotPDA)}`
          : null,
      });
    }

    // Votes
    for (const vote of proposal.votes || []) {
      if (vote.proof || vote.voteTxSignature) {
        entries.push({
          kind: "cast_vote",
          wallet: vote.proof?.wallet || vote.voter,
          signatureHex: vote.proof?.signatureHex,
          signedAt: vote.proof?.signedAt || vote.castAt,
          proof: vote.proof,
          txSignature: vote.voteTxSignature,
          onChainDetails: vote.voteTxSignature
            ? `Choice: ${vote.choiceId}`
            : null,
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
        const hasOnChain = !!entry.txSignature;

        return (
          <details key={i} className="audit-entry-details" role="listitem">
            <summary className="audit-entry">
              <span className="audit-badge">
                {badgeLabel(entry.kind)}
                {hasOnChain && <span className="audit-badge-chain"> TX</span>}
              </span>
              <span className="audit-detail">
                {compactAddress(entry.wallet)} / {formatTime(entry.signedAt)}
              </span>
              <code className="audit-sig">
                {entry.signatureHex ? compactSig(entry.signatureHex) : hasOnChain ? compactSig(entry.txSignature) : "no sig"}
              </code>
            </summary>
            <div className="audit-expanded">
              {hasOnChain && (
                <div style={{ marginBottom: 8 }}>
                  <p className="mini" style={{ marginBottom: 4 }}>
                    <strong>On-chain transaction</strong>
                    {entry.onChainDetails && <> / {entry.onChainDetails}</>}
                  </p>
                  <a
                    href={explorerTxUrl(entry.txSignature)}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="mini"
                  >
                    View on Solana Explorer: {compactSig(entry.txSignature)}
                  </a>
                </div>
              )}
              {proofJson ? (
                <>
                  <div className="audit-expanded__head">
                    <span className="mini">Wallet-signed proof</span>
                    <CopyButton text={proofJson} />
                  </div>
                  <pre className="audit-expanded__json">
                    <code>{proofJson}</code>
                  </pre>
                </>
              ) : !hasOnChain ? (
                <p className="mini">
                  No cryptographic proof attached (seeded demo data).
                </p>
              ) : null}
            </div>
          </details>
        );
      })}
    </div>
  );
}
