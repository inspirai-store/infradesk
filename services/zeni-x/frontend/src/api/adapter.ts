/**
 * API Adapter Factory
 *
 * This module creates the appropriate API adapter based on the runtime environment.
 * In Web mode, HTTP adapter is used. In Tauri mode, IPC adapter is used with
 * fallback to HTTP for modules not yet implemented.
 */

import { isTauri } from '@/utils/platform'
import { createHttpAdapter } from './adapters/http'
import { createIpcAdapter } from './adapters/ipc'
import type { IApiAdapter } from './types'

/**
 * Module-level IPC enablement configuration
 *
 * Controls which API modules use IPC when running in Tauri mode.
 * Set to true when the corresponding Rust implementation is complete.
 */
export const ipcEnabledModules = {
  connection: true,   // Connection management - implemented
  cluster: true,      // K8s cluster management - implemented
  mysql: true,        // MySQL operations - implemented
  redis: true,        // Redis operations - implemented
  history: true,      // Query history - implemented
  savedQuery: true,   // Saved queries - implemented
  k8s: true,          // K8s service discovery - implemented
  portForward: true,  // Port forwarding - implemented via kube-rs
}

// Singleton adapter instance
let _adapter: IApiAdapter | null = null

/**
 * Create API adapter based on current environment
 *
 * In Web mode: All modules use HTTP adapter
 * In Tauri mode: Uses IPC for enabled modules, HTTP fallback for others
 */
export function createApiAdapter(): IApiAdapter {
  const httpAdapter = createHttpAdapter()

  // In Web mode, always use HTTP adapter
  if (!isTauri()) {
    return httpAdapter
  }

  // In Tauri mode, create hybrid adapter
  const ipcAdapter = createIpcAdapter()

  return {
    connection: ipcEnabledModules.connection ? ipcAdapter.connection : httpAdapter.connection,
    cluster: ipcEnabledModules.cluster ? ipcAdapter.cluster : httpAdapter.cluster,
    mysql: ipcEnabledModules.mysql ? ipcAdapter.mysql : httpAdapter.mysql,
    redis: ipcEnabledModules.redis ? ipcAdapter.redis : httpAdapter.redis,
    history: ipcEnabledModules.history ? ipcAdapter.history : httpAdapter.history,
    savedQuery: ipcEnabledModules.savedQuery ? ipcAdapter.savedQuery : httpAdapter.savedQuery,
    k8s: ipcEnabledModules.k8s ? ipcAdapter.k8s : httpAdapter.k8s,
    portForward: ipcEnabledModules.portForward ? ipcAdapter.portForward : httpAdapter.portForward,
  }
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
 * Check if a specific module is using IPC
 */
export function isModuleUsingIpc(module: keyof typeof ipcEnabledModules): boolean {
  return isTauri() && ipcEnabledModules[module]
}

/**
 * Convenience export for direct API access
 * Usage: import { api } from '@/api/adapter'
 */
export const api = getApiAdapter()
