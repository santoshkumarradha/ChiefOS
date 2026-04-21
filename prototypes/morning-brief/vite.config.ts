import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  define: {
    "import.meta.env.VITE_CHIEF_CORE_URL": JSON.stringify(
      process.env.VITE_CHIEF_CORE_URL || "http://127.0.0.1:4711"
    ),
  },
});
