import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Frontier RSC features used here on STABLE Next 15: React Server Components by
  // default, streaming <Suspense> (partial render: static shell + streamed dynamic
  // holes), server actions, route handlers, and per-route Node/Edge runtimes.
  //
  // NOTE: experimental Partial Prerendering (`experimental.ppr`) requires next@canary;
  // it is intentionally NOT enabled so `next build` runs on the stable release. The
  // Suspense boundaries on the dashboard already stream the real-data tiles.
};

export default nextConfig;
