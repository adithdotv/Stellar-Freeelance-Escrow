import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// The Stellar SDK is written for Node, so the browser build needs `global`
// aliased and a Buffer polyfill.
export default defineConfig({
  plugins: [react()],
  define: {
    global: "globalThis",
  },
  // Vitest reads this block at runtime; its types ship in a separate module that
  // conflicts with rolldown-vite's plugin types, so it is declared untyped here.
  // @ts-expect-error - `test` is a Vitest extension of the Vite config.
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: "./src/test/setup.ts",
  },
});
