"use client";

import { useEffect, useState } from "react";
import { useConnection } from "@solana/wallet-adapter-react";

export default function NetworkStatus() {
  const { connection } = useConnection();
  const [slot, setSlot] = useState(null);
  const [error, setError] = useState(false);

  useEffect(() => {
    let cancelled = false;

    async function poll() {
      try {
        const s = await connection.getSlot();
        if (!cancelled) {
          setSlot(s);
          setError(false);
        }
      } catch {
        if (!cancelled) setError(true);
      }
    }

    poll();
    const id = setInterval(poll, 10_000);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [connection]);

  const label = connection.rpcEndpoint.includes("devnet") ? "Solana Devnet" : "Solana";

  return (
    <span className="network-status" aria-live="polite">
      <span className={`network-dot${error ? " network-dot--offline" : ""}`} />
      {slot != null
        ? `Slot ${slot.toLocaleString()} / ${label}`
        : error
          ? `Offline / ${label}`
          : `Connecting / ${label}`}
    </span>
  );
}
