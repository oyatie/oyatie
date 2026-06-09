import { defineConfig } from "@solidjs/start/config";

// ADR-0393: this SolidStart shell is frozen transition evidence; Leptos/Rust-WASM is canonical.
// Keep SSR enabled so archived slices remain performance-testable while migration proceeds.
export default defineConfig({
  ssr: true,
  server: {
    preset: "node-server",
    // Port 3001 keeps this archived transition shell separate from the canonical
    // Leptos render-envelope service on port 3000 during migration evidence runs.
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
