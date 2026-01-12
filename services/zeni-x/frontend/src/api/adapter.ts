/**
 * API Adapter Factory
 *
 * This module creates the IPC adapter for Tauri desktop application.
 * HTTP adapter is kept for reference but not used in production.
 */

import { createIpcAdapter } from './adapters/ipc'
import type { IApiAdapter } from './types'

// Singleton adapter instance
let _adapter: IApiAdapter | null = null

/**
 * Create API adapter
 *
 * Always uses IPC adapter for Tauri desktop application.
 */
export function createApiAdapter(): IApiAdapter {
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
