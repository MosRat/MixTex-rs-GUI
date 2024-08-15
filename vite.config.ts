import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve } from 'path';

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, "./src"),
      '@cp': resolve(__dirname, "./src/components"),
      '@view': resolve(__dirname, "./src/views"),
      '@lib': resolve(__dirname, "./src/libs"),
    }
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    rollupOptions: {
      input: {
        index: resolve(__dirname, 'index.html'),
        daemon: resolve(__dirname, 'daemon.html'),
      },
    },
    // // Tauri supports es2021
    // target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari11',
    // // don't minify for debug builds
    // minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    // // produce sourcemaps for debug builds
    // sourcemap: !!process.env.TAURI_DEBUG,
  },
}));
