import type { Metadata } from "next";
import Link from "next/link";
import "./globals.css";

export const metadata: Metadata = {
  title: "affidavit — faithful representation",
  description: "A Next.js representation that renders only the project's real data.",
};

const NAV = [
  { href: "/", label: "Dashboard" },
  { href: "/learn", label: "Learn" },
  { href: "/studio", label: "Studio" },
  { href: "/visualizer", label: "Visualizer" },
  { href: "/diff", label: "Diff" },
  { href: "/anatomy", label: "Anatomy" },
  { href: "/observe", label: "Observe" },
  { href: "/capabilities", label: "Capabilities" },
  { href: "/pipeline", label: "Pipeline" },
  { href: "/coverage", label: "Coverage" },
];

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <header className="topbar">
          <span className="brand">affidavit</span>
          <nav>
            {NAV.map((n) => (
              <Link key={n.href} href={n.href}>
                {n.label}
              </Link>
            ))}
          </nav>
          <span className="tag">faithful · no fixtures</span>
        </header>
        <main>{children}</main>
      </body>
    </html>
  );
}
