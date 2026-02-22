/// <reference types="vite/client" />

interface ImportMetaEnv {
  /**
   * API Mode for debugging
   * - 'web': Force HTTP API mode (useful for debugging without Tauri backend)
   * - 'ipc': Use IPC when available (default Tauri behavior)
   * - undefined: Auto-detect based on runtime environment
   */
  readonly VITE_API_MODE?: 'web' | 'ipc'
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
