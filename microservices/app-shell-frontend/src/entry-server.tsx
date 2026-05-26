// SSR entry point — SolidStart handles this automatically via file convention.
// Explicit export satisfies module resolution for custom adapters.
export default function handler(
  ...args: Parameters<typeof import("@solidjs/start/server").createHandler>
) {
  const { createHandler } = require("@solidjs/start/server");
  return createHandler(...args);
}
