import { defineConfig } from "@solidjs/start/config";

// ADR-0372: SolidStart SSR app-shell — supersedes Leptos prototype.
// Streaming SSR enabled per ADR-0067 §5 perf authority (SSR p99 ≤500ms).
export default defineConfig({
  ssr: true,
  server: {
    preset: "node-server",
    // Port matches Leptos prototype dev server convention (port 3001 avoids
    // conflict with the Leptos server on 3000 during parallel dev).
    port: 3001,
  },
  vite: {
    plugins: [],
    build: {
      target: "esnext",
    },
    css: {
      // Inline tokens at build time to avoid an extra network round-trip.
      preprocessorOptions: {},
    },
  },
});
