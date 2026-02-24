"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { WalletReadyState } from "@solana/wallet-adapter-base";

/** Wallet names to hide from the modal. */
const HIDDEN_WALLETS = new Set(["MetaMask"]);

/** Only show wallets that are installed or loadable. */
function isUsable(wallet) {
  return (
    wallet.readyState === WalletReadyState.Installed ||
    wallet.readyState === WalletReadyState.Loadable
  );
}

export default function WalletConnectButton() {
  const { wallets, wallet, select, connect, disconnect, connected, connecting, publicKey } =
    useWallet();
  const [open, setOpen] = useState(false);
  const [pendingConnect, setPendingConnect] = useState(false);
  const modalRef = useRef(null);

  const filtered = useMemo(
    () => wallets.filter((w) => !HIDDEN_WALLETS.has(w.adapter.name) && isUsable(w)),
    [wallets],
  );

  /* Once a wallet is selected and ready, trigger connect */
  useEffect(() => {
    if (pendingConnect && wallet && !connected && !connecting) {
      setPendingConnect(false);
      connect().catch(() => {
        /* user rejected */
      });
    }
  }, [pendingConnect, wallet, connected, connecting, connect]);

  const handleSelect = useCallback(
    (w) => {
      select(w.adapter.name);
      setPendingConnect(true);
      setOpen(false);
    },
    [select],
  );

  const handleDisconnect = useCallback(async () => {
    try {
      await disconnect();
    } catch {
      /* ignore */
    }
  }, [disconnect]);

  /* Close on Escape */
  useEffect(() => {
    if (!open) return;
    const onKey = (e) => {
      if (e.key === "Escape") setOpen(false);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open]);

  /* Close on click outside */
  useEffect(() => {
    if (!open) return;
    const onClick = (e) => {
      if (modalRef.current && !modalRef.current.contains(e.target)) {
        setOpen(false);
      }
    };
    window.addEventListener("mousedown", onClick);
    return () => window.removeEventListener("mousedown", onClick);
  }, [open]);

  /* ── Connected state ── */
  if (connected && publicKey) {
    const addr = publicKey.toBase58();
    const short = addr.slice(0, 4) + "..." + addr.slice(-4);
    return (
      <div className="wallet-connected">
        <span className="wallet-connected__addr">{short}</span>
        <button type="button" className="wallet-connected__disconnect" onClick={handleDisconnect}>
          Disconnect
        </button>
      </div>
    );
  }

  /* ── Connecting state ── */
  if (connecting) {
    return (
      <button type="button" className="button button--ghost" disabled>
        Connecting...
      </button>
    );
  }

  /* ── Default: show connect button + modal ── */
  return (
    <>
      <button type="button" className="button button--ghost" onClick={() => setOpen(true)}>
        Select Wallet
      </button>

      {open && (
        <div className="wallet-modal-overlay" aria-modal="true" role="dialog">
          <div className="wallet-modal" ref={modalRef}>
            <div className="wallet-modal__header">
              <span className="wallet-modal__title">Connect Wallet</span>
              <button
                type="button"
                className="wallet-modal__close"
                onClick={() => setOpen(false)}
                aria-label="Close"
              >
                <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
                  <path
                    d="M1 1L13 13M13 1L1 13"
                    stroke="currentColor"
                    strokeWidth="1.5"
                    strokeLinecap="round"
                  />
                </svg>
              </button>
            </div>

            <div className="wallet-modal__body">
              {filtered.length === 0 ? (
                <div className="wallet-modal__empty">
                  <p>No Solana wallets detected.</p>
                  <a
                    href="https://phantom.app/"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="wallet-modal__install"
                  >
                    Install Phantom &rarr;
                  </a>
                </div>
              ) : (
                <ul className="wallet-modal__list">
                  {filtered.map((wallet) => (
                    <li key={wallet.adapter.name}>
                      <button
                        type="button"
                        className="wallet-modal__option"
                        onClick={() => handleSelect(wallet)}
                      >
                        {wallet.adapter.icon && (
                          <img
                            src={wallet.adapter.icon}
                            alt=""
                            width={28}
                            height={28}
                            className="wallet-modal__icon"
                          />
                        )}
                        <span className="wallet-modal__name">{wallet.adapter.name}</span>
                        {wallet.readyState === WalletReadyState.Installed && (
                          <span className="wallet-modal__detected">Detected</span>
                        )}
                      </button>
                    </li>
                  ))}
                </ul>
              )}
            </div>

            <div className="wallet-modal__footer">
              <span className="mini">Solana Devnet</span>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
