"use client";

import { useEffect } from "react";

/**
 * MetaMask and other wallet extensions inject scripts into every page. When they
 * fail (e.g. "Failed to connect to MetaMask"), that can surface as an unhandled
 * error. This app uses Phantom only; we suppress those extension errors so they
 * don't trigger the Next.js error overlay.
 */
export default function SuppressExtensionErrors() {
  useEffect(() => {
    function handleRejection(event) {
      const msg =
        typeof event.reason === "string"
          ? event.reason
          : event.reason?.message ?? "";
      if (
        /MetaMask|Failed to connect|chrome-extension:/i.test(msg) &&
        !/phantom/i.test(msg)
      ) {
        event.preventDefault();
        console.info(
          "[Ballot Guardian] Ignoring wallet extension error (this app uses Phantom only):",
          msg.slice(0, 80)
        );
      }
    }

    window.addEventListener("unhandledrejection", handleRejection);
    return () => window.removeEventListener("unhandledrejection", handleRejection);
  }, []);

  return null;
}
