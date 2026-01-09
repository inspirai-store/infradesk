import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

// Check if running in Tauri development mode
const isTauriDev = process.env.TAURI_ENV_DEBUG !== undefined

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  // Prevent Vite from clearing the terminal to allow Tauri to output
  clearScreen: false,
  server: {
    port: 15073,
    // Make sure the server is accessible when running in Tauri
    host: process.env.TAURI_DEV_HOST || 'localhost',
    // Enable HMR in Tauri
    strictPort: true,
    // Only enable proxy for web mode (when not in Tauri)
    // In Tauri mode, API calls go through Rust IPC, not HTTP
    ...(isTauriDev ? {} : {
      proxy: {
        '/api': {
          target: 'http://localhost:15080',
          changeOrigin: true,
        },
      },
    }),
  },
  // Environment variable prefix to expose to the client
  envPrefix: ['VITE_', 'TAURI_ENV_'],
  build: {
    outDir: 'dist',
    sourcemap: false,
    // Tauri uses Chromium on Windows and WebKit on macOS/Linux
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari15',
  },
})

