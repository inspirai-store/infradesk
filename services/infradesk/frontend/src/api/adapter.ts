/**
 * API Adapter Factory
 *
 * This module creates the appropriate API adapter based on runtime environment:
 * - Tauri desktop: IPC adapter (direct Tauri commands)
 * - Web browser: HTTP adapter (via HTTP API at /api)
 *
 * Environment detection uses `@/utils/platform` which checks for Tauri runtime
 * and can be overridden with VITE_API_MODE environment variable.
 */

import { isTauri } from '@/utils/platform'
import { createHttpAdapter } from './adapters/http'
import { createIpcAdapter } from './adapters/ipc'
import type { IApiAdapter } from './types'

// Singleton adapter instance
let _adapter: IApiAdapter | null = null

/**
 * Create API adapter based on runtime environment
 *
 * - In Tauri desktop: uses IPC adapter for direct command invocation
 * - In Web browser: uses HTTP adapter with Vite proxy to backend
 */
export function createApiAdapter(): IApiAdapter {
  // Web mode (browser or forced via VITE_API_MODE=web): use HTTP adapter
  if (!isTauri()) {
    console.log('[API] Using HTTP adapter (web mode)')
    return createHttpAdapter()
  }

  // Tauri desktop mode: use IPC adapter
  console.log('[API] Using IPC adapter (Tauri mode)')
  return createIpcAdapter()
}

/**
 * Get singleton API adapter instance
 *
 * Creates the adapter on first call, returns cached instance thereafter.
 */
export function getApiAdapter(): IApiAdapter {
  if (!_adapter) {
    _adapter = createApiAdapter()
  }
  return _adapter
}

/**
 * Reset adapter instance (useful for testing)
 */
export function resetApiAdapter(): void {
  _adapter = null
}

/**
 * Convenience export for direct API access
 * Usage: import { api } from '@/api/adapter'
 */
export const api = getApiAdapter()
