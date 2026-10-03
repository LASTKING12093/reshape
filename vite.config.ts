import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1424,
    strictPort: true,
    watch: {
      ignored: [
        "**/src-tauri/**",
        "**/.cache/**",
        "**/test-output/**",
        "**/release/**",
      ],
    },
  },
  build: { target: "es2022" },
});
