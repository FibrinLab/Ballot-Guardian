import "./globals.css";

export const metadata = {
  title: "Ballot Guardian | Realms Governance Extension",
  description: "Reputation-weighted quadratic voting for Realms DAOs on Solana.",
};

export default function RootLayout({ children }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}

