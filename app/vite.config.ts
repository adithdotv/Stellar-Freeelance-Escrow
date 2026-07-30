import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// The Stellar SDK is written for Node, so the browser build needs `global`
// aliased and a Buffer polyfill.
export default defineConfig({
  plugins: [react()],
  define: {
    global: "globalThis",
  },
});
