var __assign = (this && this.__assign) || function () {
    __assign = Object.assign || function(t) {
        for (var s, i = 1, n = arguments.length; i < n; i++) {
            s = arguments[i];
            for (var p in s) if (Object.prototype.hasOwnProperty.call(s, p))
                t[p] = s[p];
        }
        return t;
    };
    return __assign.apply(this, arguments);
};
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import { resolve } from 'path';
// Check if running in Tauri development mode
var isTauriDev = process.env.TAURI_ENV_DEBUG !== undefined;
export default defineConfig({
    plugins: [vue()],
    resolve: {
        alias: {
            '@': resolve(__dirname, 'src'),
        },
    },
    // Prevent Vite from clearing the terminal to allow Tauri to output
    clearScreen: false,
    server: __assign({ port: 15073, 
        // Make sure the server is accessible when running in Tauri
        host: process.env.TAURI_DEV_HOST || 'localhost', 
        // Enable HMR in Tauri
        strictPort: true }, (isTauriDev ? {} : {
        proxy: {
            '/api': {
                target: 'http://localhost:15080',
                changeOrigin: true,
            },
        },
    })),
    // Environment variable prefix to expose to the client
    envPrefix: ['VITE_', 'TAURI_ENV_'],
    build: {
        outDir: 'dist',
        sourcemap: false,
        // Tauri uses Chromium on Windows and WebKit on macOS/Linux
        target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari15',
    },
});
