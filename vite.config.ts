import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import packageInfo from "./package.json" with { type: "json" };

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  define: { __APP_VERSION__: JSON.stringify(packageInfo.version) },
});
