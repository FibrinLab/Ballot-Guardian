"use client";

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
    });
  }

  for (const proposal of dao.proposals || []) {
    if (proposal.proof) {
      entries.push({
        kind: "create_ballot",
        wallet: proposal.proof.wallet,
        signatureHex: proposal.proof.signatureHex,
        signedAt: proposal.proof.signedAt,
      });
    }
    for (const vote of proposal.votes || []) {
      if (vote.proof) {
        entries.push({
          kind: "cast_vote",
          wallet: vote.proof.wallet,
          signatureHex: vote.proof.signatureHex,
          signedAt: vote.proof.signedAt,
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
      {entries.map((entry, i) => (
        <div key={i} className="audit-entry" role="listitem">
          <span className="audit-badge">{badgeLabel(entry.kind)}</span>
          <span className="audit-detail">
            {compactAddress(entry.wallet)} / {formatTime(entry.signedAt)}
          </span>
          <code className="audit-sig">{compactSig(entry.signatureHex)}</code>
        </div>
      ))}
    </div>
  );
}
