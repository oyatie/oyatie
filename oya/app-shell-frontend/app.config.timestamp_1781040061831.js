// app.config.ts
import { defineConfig } from "@solidjs/start/config";
var app_config_default = defineConfig({
  ssr: true,
  server: {
    preset: "node-server",
    // Port 3001 keeps this archived transition shell separate from the canonical
    // Leptos render-envelope service on port 3000 during migration evidence runs.
    port: 3001
  },
  vite: {
    plugins: [],
    build: {
      target: "esnext"
    },
    css: {
      // Inline tokens at build time to avoid an extra network round-trip.
      preprocessorOptions: {}
    }
  }
});
export {
  app_config_default as default
};
