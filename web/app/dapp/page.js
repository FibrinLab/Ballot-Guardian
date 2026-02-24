"use client";

import Link from "next/link";
import { startTransition, useCallback, useDeferredValue, useEffect, useState } from "react";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import WalletConnectButton from "../components/WalletConnectButton";
import ReputationDashboard from "../components/ReputationDashboard";
import AuditTrail from "../components/AuditTrail";
import {
  castVote,
  createEmptyDemoStore,
  createProposal,
  getDaoById,
  getProposalTallies,
  getWalletVote,
  isProposalClosed,
  loadDemoStore,
  registerDao,
  saveDemoStore,
} from "../../lib/demoStore";

function useCountdown(closesAt) {
  const compute = useCallback(() => {
    const now = Date.now();
    const end = new Date(closesAt).getTime();
    const diff = end - now;
    if (diff <= 0) return null;
    const hours = Math.floor(diff / (1000 * 60 * 60));
    const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));
    return `${hours}h ${minutes}m`;
  }, [closesAt]);

  const [remaining, setRemaining] = useState(compute);

  useEffect(() => {
    setRemaining(compute());
    const id = setInterval(() => setRemaining(compute()), 60_000);
    return () => clearInterval(id);
  }, [compute]);

  return remaining;
}

function CountdownLabel({ closesAt, closed }) {
  const remaining = useCountdown(closesAt);
  if (closed || !remaining) {
    return <>Closed {formatDate(closesAt)}</>;
  }
  return <>Closes in {remaining}</>;
}

function VoteBar({ tallies }) {
  const total = tallies.yes + tallies.no + tallies.abstain;
  if (total === 0) return null;
  const yesPct = (tallies.yes / total) * 100;
  const noPct = (tallies.no / total) * 100;
  const abstainPct = (tallies.abstain / total) * 100;
  return (
    <div className="vote-bar" aria-label="Vote distribution">
      {tallies.yes > 0 && (
        <div
          className="vote-bar__segment vote-bar__segment--yes"
          style={{ width: `${yesPct}%` }}
        >
          {yesPct >= 12 && <span>{Math.round(yesPct)}%</span>}
        </div>
      )}
      {tallies.no > 0 && (
        <div
          className="vote-bar__segment vote-bar__segment--no"
          style={{ width: `${noPct}%` }}
        >
          {noPct >= 12 && <span>{Math.round(noPct)}%</span>}
        </div>
      )}
      {tallies.abstain > 0 && (
        <div
          className="vote-bar__segment vote-bar__segment--abstain"
          style={{ width: `${abstainPct}%` }}
        >
          {abstainPct >= 12 && <span>{Math.round(abstainPct)}%</span>}
        </div>
      )}
    </div>
  );
}

function QvCostIndicator({ proposal }) {
  const totalVotes = proposal.votes?.length || 0;
  const creditsSpent = totalVotes * totalVotes;
  const nextCost = 2 * totalVotes + 1;
  return (
    <div className="qv-cost-indicator">
      <span className="mini">QUADRATIC COST</span>
      <div className="qv-cost-indicator__values">
        <span>
          Votes cast: <strong>{totalVotes}</strong>
        </span>
        <span>
          Credits spent: <strong>{creditsSpent}</strong>
        </span>
        <span>
          Next vote: <strong>{nextCost} credit{nextCost !== 1 ? "s" : ""}</strong>
        </span>
      </div>
    </div>
  );
}

export default function DappPage() {
  const { connection } = useConnection();
  const { connected, publicKey, signMessage, wallet } = useWallet();

  const [store, setStore] = useState(createEmptyDemoStore);
  const [hydrated, setHydrated] = useState(false);
  const [mounted, setMounted] = useState(false);
  const [selectedDaoId, setSelectedDaoId] = useState(null);
  const [balanceLamports, setBalanceLamports] = useState(null);
  const [status, setStatus] = useState(null);
  const [busyAction, setBusyAction] = useState("");

  const [daoFilterInput, setDaoFilterInput] = useState("");
  const deferredDaoFilter = useDeferredValue(daoFilterInput);

  const [daoForm, setDaoForm] = useState({
    name: "",
    description: "",
  });

  const [proposalForm, setProposalForm] = useState({
    question: "",
    description: "",
    durationHours: "24",
  });

  const walletAddress = publicKey?.toBase58() || "";
  const selectedDao = getDaoById(store, selectedDaoId);
  const filteredDaos = store.daos.filter((dao) => {
    const q = deferredDaoFilter.trim().toLowerCase();
    if (!q) return true;
    return (
      dao.name.toLowerCase().includes(q) ||
      (dao.description || "").toLowerCase().includes(q) ||
      (dao.slug || "").toLowerCase().includes(q)
    );
  });

  useEffect(() => {
    setMounted(true);
  }, []);

  useEffect(() => {
    const nextStore = loadDemoStore();
    setStore(nextStore);
    setSelectedDaoId(nextStore.daos[0]?.id || null);
    setHydrated(true);
  }, []);

  useEffect(() => {
    if (!hydrated) return;
    saveDemoStore(store);

    if (selectedDaoId && !store.daos.some((dao) => dao.id === selectedDaoId)) {
      setSelectedDaoId(store.daos[0]?.id || null);
    }
  }, [hydrated, selectedDaoId, store]);

  useEffect(() => {
    let cancelled = false;

    async function fetchBalance() {
      if (!publicKey) {
        setBalanceLamports(null);
        return;
      }

      try {
        const lamports = await connection.getBalance(publicKey);
        if (!cancelled) setBalanceLamports(lamports);
      } catch {
        if (!cancelled) setBalanceLamports(null);
      }
    }

    fetchBalance();

    return () => {
      cancelled = true;
    };
  }, [connection, publicKey]);

  async function refreshBalance() {
    if (!publicKey) return;
    try {
      const lamports = await connection.getBalance(publicKey);
      setBalanceLamports(lamports);
      setStatus({ tone: "ok", text: "Wallet balance refreshed." });
    } catch (error) {
      setStatus({ tone: "error", text: toErrorText(error, "Could not refresh balance.") });
    }
  }

  async function signActionProof(kind, payload) {
    if (!connected || !publicKey) {
      throw new Error("Connect a wallet first.");
    }
    if (!signMessage) {
      throw new Error("This wallet does not support message signing.");
    }

    const message = JSON.stringify({
      app: "Ballot Guardian",
      kind,
      wallet: publicKey.toBase58(),
      ts: new Date().toISOString(),
      payload,
    });

    const signature = await signMessage(new TextEncoder().encode(message));

    return {
      message,
      signatureHex: toHex(signature),
      wallet: publicKey.toBase58(),
      signedAt: new Date().toISOString(),
    };
  }

  async function handleRegisterDao(event) {
    event.preventDefault();

    try {
      setBusyAction("register-dao");
      setStatus(null);

      const proof = await signActionProof("register_dao", {
        name: daoForm.name,
        description: daoForm.description,
      });

      const result = registerDao(store, {
        ...daoForm,
        createdBy: walletAddress,
        proof,
      });

      startTransition(() => {
        setStore(result.store);
        setSelectedDaoId(result.dao.id);
      });

      setDaoForm({ name: "", description: "" });
      setStatus({ tone: "ok", text: `Registered union/DAO: ${result.dao.name}` });
    } catch (error) {
      setStatus({ tone: "error", text: toErrorText(error, "Could not register union/DAO.") });
    } finally {
      setBusyAction("");
    }
  }

  async function handleCreateProposal(event) {
    event.preventDefault();

    try {
      setBusyAction("create-proposal");
      setStatus(null);

      const proof = await signActionProof("create_ballot", {
        daoId: selectedDaoId,
        question: proposalForm.question,
        durationHours: proposalForm.durationHours,
      });

      const result = createProposal(store, {
        daoId: selectedDaoId,
        question: proposalForm.question,
        description: proposalForm.description,
        durationHours: proposalForm.durationHours,
        createdBy: walletAddress,
        proof,
      });

      startTransition(() => setStore(result.store));
      setProposalForm({ question: "", description: "", durationHours: "24" });
      setStatus({ tone: "ok", text: "Ballot created." });
    } catch (error) {
      setStatus({ tone: "error", text: toErrorText(error, "Could not create ballot.") });
    } finally {
      setBusyAction("");
    }
  }

  async function handleVote(proposal, choiceId) {
    try {
      setBusyAction(`vote-${proposal.id}`);
      setStatus(null);

      const proof = await signActionProof("cast_vote", {
        daoId: selectedDaoId,
        proposalId: proposal.id,
        choiceId,
      });

      const result = castVote(store, {
        daoId: selectedDaoId,
        proposalId: proposal.id,
        voter: walletAddress,
        choiceId,
        proof,
      });

      startTransition(() => setStore(result.store));
      setStatus({ tone: "ok", text: `Vote recorded: ${choiceLabel(choiceId)}` });
    } catch (error) {
      setStatus({ tone: "error", text: toErrorText(error, "Vote failed.") });
    } finally {
      setBusyAction("");
    }
  }

  return (
    <>
      <div className="scanline" aria-hidden="true" />

      <header className="topbar">
        <div className="topbar__inner">
          <Link href="/" className="topbar__brand">BALLOT GUARDIAN / DAPP</Link>
          <nav aria-label="Primary">
            <Link href="/whitepaper">White Paper</Link>
          </nav>
        </div>
      </header>

      <main className="page dapp-shell">
        <section className="hero dapp-hero">
          <p className="kicker">GOVERNANCE DEMO / SOLANA DEVNET</p>
          <h1>Register unions and run wallet-backed ballots</h1>
          <p className="lead">
            Connect your Solana wallet, register organizations, create ballots, and cast
            wallet-signed votes. All actions are cryptographically signed and auditable.
          </p>

          <div className="dapp-hero__actions">
            <WalletConnectButton />
            <button
              type="button"
              className="button button--ghost"
              onClick={refreshBalance}
              disabled={!connected}
            >
              Refresh Balance
            </button>
          </div>

          <div className="wallet-grid">
            <div className="subpanel">
              <p className="label">WALLET</p>
              <p className="mono-value">
                {walletAddress ? compactAddress(walletAddress) : "Not connected"}
              </p>
              <p className="mini" suppressHydrationWarning>
                {mounted ? (wallet?.adapter?.name || "No wallet selected") : "No wallet selected"}
              </p>
            </div>
            <div className="subpanel">
              <p className="label">SIGN MESSAGE</p>
              <p className="mono-value">{signMessage ? "Supported" : "Unsupported"}</p>
              <p className="mini">Required for registering unions, creating ballots, and voting.</p>
            </div>
            <div className="subpanel">
              <p className="label">NETWORK / BALANCE</p>
              <p className="mono-value">
                {connection.rpcEndpoint.includes("devnet") ? "Devnet" : "Custom RPC"}
              </p>
              <p className="mini">
                {balanceLamports == null ? "Balance unavailable" : `${lamportsToSol(balanceLamports)} SOL`}
              </p>
            </div>
          </div>

          {status ? (
            <p className="status-banner" data-tone={status.tone} role="status">
              {status.text}
            </p>
          ) : null}
        </section>

        <section className="dapp-grid" aria-label="Dapp workspace">
          <section className="panel dapp-column" aria-labelledby="dao-section-title">
            <div className="panel__head">
              <p className="label">UNIONS / DAOS</p>
              <p className="mini">{store.daos.length} registered</p>
            </div>
            <h2 id="dao-section-title">Register organization</h2>

            <form className="stack" onSubmit={handleRegisterDao}>
              <label className="field">
                <span className="field__label">Union / DAO name</span>
                <input
                  type="text"
                  value={daoForm.name}
                  onChange={(event) =>
                    setDaoForm((prev) => ({ ...prev, name: event.target.value }))
                  }
                  placeholder="e.g. Junior Doctors Union"
                  required
                  disabled={!connected || busyAction === "register-dao"}
                />
              </label>

              <label className="field">
                <span className="field__label">Description</span>
                <textarea
                  value={daoForm.description}
                  onChange={(event) =>
                    setDaoForm((prev) => ({ ...prev, description: event.target.value }))
                  }
                  placeholder="Short summary of the organization and its voting scope."
                  rows={4}
                  disabled={!connected || busyAction === "register-dao"}
                />
              </label>

              <button
                className="button"
                type="submit"
                disabled={!connected || busyAction === "register-dao"}
              >
                {busyAction === "register-dao" ? "Signing + Registering..." : "Register Union"}
              </button>
            </form>

            <div className="list-tools">
              <label className="field">
                <span className="field__label">Search organizations</span>
                <input
                  type="search"
                  value={daoFilterInput}
                  onChange={(event) => setDaoFilterInput(event.target.value)}
                  placeholder="Filter by name or description"
                />
              </label>
            </div>

            <div className="card-stack" role="list" aria-label="Registered unions">
              {filteredDaos.length === 0 ? (
                <div className="empty-state">
                  {store.daos.length === 0
                    ? "No unions registered yet."
                    : "No organizations match your search."}
                </div>
              ) : (
                filteredDaos.map((dao) => {
                  const isSelected = dao.id === selectedDaoId;
                  return (
                    <button
                      key={dao.id}
                      type="button"
                      className={`dao-card${isSelected ? " is-active" : ""}`}
                      onClick={() => setSelectedDaoId(dao.id)}
                    >
                      <span className="dao-card__name">{dao.name}</span>
                      <span className="dao-card__meta">
                        {dao.proposals?.length || 0} ballots / {compactAddress(dao.createdBy)}
                      </span>
                    </button>
                  );
                })
              )}
            </div>
          </section>

          <section className="panel workspace-column" aria-labelledby="workspace-title">
            <div className="panel__head">
              <p className="label">BALLOTS</p>
              <p className="mini">
                {selectedDao ? `${selectedDao.proposals?.length || 0} active/history items` : "Select a union"}
              </p>
            </div>

            {selectedDao ? (
              <>
                <h2 id="workspace-title">{selectedDao.name}</h2>
                <p className="mini">
                  {selectedDao.description || "No description provided."}
                </p>

                <div className="subpanel subpanel--spaced">
                  <p className="label">CREATE BALLOT</p>
                  <form className="stack" onSubmit={handleCreateProposal}>
                    <label className="field">
                      <span className="field__label">Question</span>
                      <input
                        type="text"
                        value={proposalForm.question}
                        onChange={(event) =>
                          setProposalForm((prev) => ({ ...prev, question: event.target.value }))
                        }
                        placeholder="Should the union adopt policy X?"
                        required
                        disabled={!connected || busyAction === "create-proposal"}
                      />
                    </label>

                    <label className="field">
                      <span className="field__label">Details (optional)</span>
                      <textarea
                        value={proposalForm.description}
                        onChange={(event) =>
                          setProposalForm((prev) => ({ ...prev, description: event.target.value }))
                        }
                        placeholder="Context, scope, dates, implementation notes."
                        rows={4}
                        disabled={!connected || busyAction === "create-proposal"}
                      />
                    </label>

                    <label className="field">
                      <span className="field__label">Voting window (hours)</span>
                      <input
                        type="number"
                        min="1"
                        max="720"
                        step="1"
                        value={proposalForm.durationHours}
                        onChange={(event) =>
                          setProposalForm((prev) => ({
                            ...prev,
                            durationHours: event.target.value,
                          }))
                        }
                        disabled={!connected || busyAction === "create-proposal"}
                      />
                    </label>

                    <button
                      className="button"
                      type="submit"
                      disabled={!connected || busyAction === "create-proposal"}
                    >
                      {busyAction === "create-proposal" ? "Signing + Creating..." : "Create Ballot"}
                    </button>
                  </form>
                </div>

                <div className="card-stack">
                  {(selectedDao.proposals || []).length === 0 ? (
                    <div className="empty-state">No ballots yet for this union.</div>
                  ) : (
                    selectedDao.proposals.map((proposal) => {
                      const tallies = getProposalTallies(proposal);
                      const walletVote = getWalletVote(proposal, walletAddress);
                      const closed = isProposalClosed(proposal);

                      return (
                        <article key={proposal.id} className="proposal-card">
                          <div className="proposal-card__head">
                            <div>
                              <h3>{proposal.question}</h3>
                              <p className="mini">
                                Created {formatDate(proposal.createdAt)} /{" "}
                                <CountdownLabel closesAt={proposal.closesAt} closed={closed} />
                              </p>
                            </div>
                            <span className={`pill${closed ? " pill--muted" : ""}`}>
                              {closed ? "Closed" : "Open"}
                            </span>
                          </div>

                          {proposal.description ? <p>{proposal.description}</p> : null}

                          <div className="vote-tallies" aria-label="Vote tallies">
                            <div>
                              <span className="mini">Yes</span>
                              <strong>{tallies.yes}</strong>
                            </div>
                            <div>
                              <span className="mini">No</span>
                              <strong>{tallies.no}</strong>
                            </div>
                            <div>
                              <span className="mini">Abstain</span>
                              <strong>{tallies.abstain}</strong>
                            </div>
                            <div>
                              <span className="mini">Votes</span>
                              <strong>{proposal.votes?.length || 0}</strong>
                            </div>
                          </div>

                          <VoteBar tallies={tallies} />

                          <div className="vote-grid" role="group" aria-label={`Vote options for ${proposal.question}`}>
                            {(proposal.choices || []).map((choice) => (
                              <button
                                key={choice.id}
                                type="button"
                                className={`vote-button${
                                  walletVote?.choiceId === choice.id ? " is-selected" : ""
                                }`}
                                disabled={!connected || closed || busyAction === `vote-${proposal.id}`}
                                onClick={() => handleVote(proposal, choice.id)}
                              >
                                {choice.label}
                              </button>
                            ))}
                          </div>

                          <QvCostIndicator proposal={proposal} />

                          <p className="mini proposal-card__foot">
                            {walletVote
                              ? `Your vote: ${choiceLabel(walletVote.choiceId)} (${formatDate(walletVote.castAt)})`
                              : "You have not voted on this ballot yet."}
                          </p>
                        </article>
                      );
                    })
                  )}
                </div>
              </>
            ) : (
              <>
                <h2 id="workspace-title">No union selected</h2>
                <div className="empty-state">
                  Register a union (DAO) on the left, or select one from the list to create ballots
                  and vote.
                </div>
              </>
            )}
          </section>
        </section>

        <section className="panel" aria-labelledby="rep-dashboard-title">
          <div className="panel__head">
            <p className="label">REPUTATION ENGINE (INTERACTIVE)</p>
          </div>
          <h2 id="rep-dashboard-title">Voter reputation profile</h2>
          <p className="mini" style={{ marginBottom: 12 }}>
            Drag the sliders to explore how a voter&#39;s reputation multiplier is computed from
            five on-chain scoring components.
          </p>
          <ReputationDashboard />
        </section>

        <section className="panel" aria-labelledby="audit-trail-title">
          <div className="panel__head">
            <p className="label">AUDIT TRAIL</p>
            <p className="mini">All wallet-signed actions for the selected DAO</p>
          </div>
          <h2 id="audit-trail-title">On-chain auditability</h2>
          <AuditTrail store={store} daoId={selectedDaoId} />
        </section>
      </main>
    </>
  );
}

function toErrorText(error, fallback) {
  if (!error) return fallback;
  if (typeof error === "string") return error;
  if (error instanceof Error && error.message) return error.message;
  return fallback;
}

function toHex(uint8Array) {
  return Array.from(uint8Array, (byte) => byte.toString(16).padStart(2, "0")).join("");
}

function compactAddress(value) {
  if (!value) return "n/a";
  if (value.length < 10) return value;
  return `${value.slice(0, 4)}...${value.slice(-4)}`;
}

function lamportsToSol(lamports) {
  return (lamports / 1_000_000_000).toFixed(4);
}

function formatDate(isoString) {
  const date = new Date(isoString);
  if (Number.isNaN(date.getTime())) return "invalid date";
  return date.toLocaleString();
}

function choiceLabel(choiceId) {
  if (choiceId === "yes") return "Yes";
  if (choiceId === "no") return "No";
  if (choiceId === "abstain") return "Abstain";
  return choiceId;
}
