import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

// Read port from environment variable, default to 15073
const serverPort = parseInt(process.env.VITE_PORT || '15073', 10)

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
    port: serverPort,
    // Make sure the server is accessible when running in Tauri
    host: process.env.TAURI_DEV_HOST || 'localhost',
    // Enable HMR in Tauri
    strictPort: true,
    // Proxy /api to Rust HTTP backend for web debug mode
    // In Tauri IPC mode this proxy is not used (API calls go through IPC)
    // In web browser mode (VITE_API_MODE=web) this proxy routes to backend
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:12420',
        changeOrigin: true,
        secure: false,
        configure: (proxy, _options) => {
          proxy.on('error', (err, _req, _res) => {
            console.log('proxy error', err)
          })
          proxy.on('proxyReq', (proxyReq, req, _res) => {
            console.log('Sending Request:', req.method, req.url)
          })
          proxy.on('proxyRes', (proxyRes, req, _res) => {
            console.log('Received Response:', proxyRes.statusCode, req.url)
          })
        },
      },
    },
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

