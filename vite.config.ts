import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const host = process.env.TAURI_DEV_HOST;
const externalOutDir = process.env.WORKLORE_FRONTEND_DIST;

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  build: {
    outDir: externalOutDir || "dist",
    emptyOutDir: true,
  },
  server: {
    port: 1427,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1428,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
