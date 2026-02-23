import "./globals.css";
import "@solana/wallet-adapter-react-ui/styles.css";
import AppProviders from "./providers";

export const metadata = {
  title: "Ballot Guardian | Realms Governance Extension",
  description: "Reputation-weighted quadratic voting for Realms DAOs on Solana.",
};

export default function RootLayout({ children }) {
  return (
    <html lang="en">
      <body>
        <AppProviders>{children}</AppProviders>
      </body>
    </html>
  );
}
