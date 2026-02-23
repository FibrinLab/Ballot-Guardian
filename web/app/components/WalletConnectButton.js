"use client";

import dynamic from "next/dynamic";

const WalletMultiButton = dynamic(
  async () => (await import("@solana/wallet-adapter-react-ui")).WalletMultiButton,
  {
    ssr: false,
    loading: () => (
      <button type="button" className="button button--ghost" disabled>
        Loading Wallet…
      </button>
    ),
  },
);

export default function WalletConnectButton() {
  return <WalletMultiButton className="wallet-adapter-button" />;
}

