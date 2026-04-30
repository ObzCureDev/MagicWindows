import { defineConfig } from "vitest/config";

// Separate from vite.config.ts so the Svelte plugin (which crashes Vitest's
// dev-server bootstrap) is not loaded for unit tests. The tests in this
// workspace are pure TS modules; no Svelte component compilation is needed.
export default defineConfig({
  test: {
    include: ["src/**/*.test.ts"],
  },
});
