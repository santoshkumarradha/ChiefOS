import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

/**
 * Dev server proxy: forward `/v1/*` to the chief-core HTTP API.
 *
 * Production builds hit same-origin `/v1/*`, so only dev needs a proxy.
 * Override the backend target via VITE_CHIEF_CORE_URL:
 *   VITE_CHIEF_CORE_URL=http://127.0.0.1:4711 npm run dev
 */
const BACKEND_URL =
  process.env.VITE_CHIEF_CORE_URL || "http://localhost:8080";

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      "/v1": {
        target: BACKEND_URL,
        changeOrigin: true,
        ws: false,
      },
    },
  },
  define: {
    "import.meta.env.VITE_CHIEF_CORE_URL": JSON.stringify(
      process.env.VITE_CHIEF_CORE_URL || ""
    ),
  },
});
