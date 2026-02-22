/**
 * Platform Detection Utility
 *
 * This module provides runtime environment detection for determining whether
 * the application is running in a Tauri desktop environment or a web browser.
 *
 * Usage:
 *   import { isTauri, isWeb, getPlatformInfo } from '@/utils/platform'
 *
 *   if (isTauri()) {
 *     // Use Tauri IPC commands
 *     const result = await invoke('my_command')
 *   } else {
 *     // Use HTTP API
 *     const result = await axios.get('/api/my-endpoint')
 *   }
 */

/**
 * Platform information interface
 */
export interface PlatformInfo {
  /** Whether running in Tauri desktop app */
  isTauri: boolean
  /** Whether running in web browser */
  isWeb: boolean
  /** Operating system (only available in Tauri) */
  os?: 'windows' | 'macos' | 'linux' | 'ios' | 'android' | 'unknown'
  /** Platform arch (only available in Tauri) */
  arch?: string
  /** Tauri version (only available in Tauri) */
  tauriVersion?: string
  /** Webview version (only available in Tauri) */
  webviewVersion?: string
}

// Cache the Tauri detection result
let _isTauriCached: boolean | null = null

/**
 * Check if API mode is forced via environment variable.
 *
 * VITE_API_MODE can be:
 * - 'web': Force HTTP API mode (useful for debugging without Tauri)
 * - 'ipc': Use IPC when available (default Tauri behavior)
 * - undefined: Auto-detect based on runtime environment
 */
function isForceWebMode(): boolean {
  return import.meta.env.VITE_API_MODE === 'web'
}

/**
 * Check if the application is running in a Tauri desktop environment.
 *
 * This function checks for the presence of the `__TAURI_INTERNALS__` object
 * which is injected by Tauri into the webview context.
 *
 * Note: If VITE_API_MODE=web is set, this will always return false to force
 * HTTP API usage for debugging purposes.
 *
 * @returns true if running in Tauri, false if running in web browser
 *
 * @example
 * ```ts
 * import { isTauri } from '@/utils/platform'
 *
 * if (isTauri()) {
 *   // Tauri-specific code
 *   console.log('Running in Tauri desktop app')
 * } else {
 *   // Web browser-specific code
 *   console.log('Running in web browser')
 * }
 * ```
 */
export function isTauri(): boolean {
  // Check for forced web mode first
  if (isForceWebMode()) {
    return false
  }

  if (_isTauriCached !== null) {
    return _isTauriCached
  }

  // Check for Tauri 2.x API (uses __TAURI_INTERNALS__)
  // This is the recommended way to detect Tauri in version 2.x
  if (typeof window !== 'undefined') {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const hasInternals = !!(window as any).__TAURI_INTERNALS__
    // Also check for older 1.x API for compatibility
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const hasIpc = !!(window as any).__TAURI_IPC__
    // Check for Tauri metadata (available in both versions)
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const hasTauriMeta = !!(window as any).__TAURI_METADATA__

    _isTauriCached = hasInternals || hasIpc || hasTauriMeta
    return _isTauriCached
  }

  _isTauriCached = false
  return false
}

/**
 * Check if the application is running in a web browser environment.
 *
 * This is the inverse of `isTauri()`.
 *
 * @returns true if running in web browser, false if running in Tauri
 *
 * @example
 * ```ts
 * import { isWeb } from '@/utils/platform'
 *
 * if (isWeb()) {
 *   // Use HTTP-based API calls
 *   await fetch('/api/endpoint')
 * }
 * ```
 */
export function isWeb(): boolean {
  return !isTauri()
}

/**
 * Get detailed platform information.
 *
 * In Tauri environment, this includes OS, architecture, and version info.
 * In web browser, only basic environment flags are returned.
 *
 * Note: This is a synchronous function that returns immediately available info.
 * For async platform details (like OS info), use `getPlatformInfoAsync()`.
 *
 * @returns Platform information object
 *
 * @example
 * ```ts
 * import { getPlatformInfo } from '@/utils/platform'
 *
 * const info = getPlatformInfo()
 * console.log(info.isTauri ? 'Desktop' : 'Web')
 * ```
 */
export function getPlatformInfo(): PlatformInfo {
  const tauriDetected = isTauri()

  return {
    isTauri: tauriDetected,
    isWeb: !tauriDetected,
  }
}

/**
 * Get detailed platform information asynchronously.
 *
 * In Tauri environment, this fetches additional OS and version information
 * using Tauri's OS plugin APIs.
 *
 * @returns Promise resolving to platform information object
 *
 * @example
 * ```ts
 * import { getPlatformInfoAsync } from '@/utils/platform'
 *
 * const info = await getPlatformInfoAsync()
 * if (info.isTauri) {
 *   console.log(`Running on ${info.os} (${info.arch})`)
 *   console.log(`Tauri version: ${info.tauriVersion}`)
 * }
 * ```
 */
export async function getPlatformInfoAsync(): Promise<PlatformInfo> {
  const basicInfo = getPlatformInfo()

  if (!basicInfo.isTauri) {
    return basicInfo
  }

  try {
    // Dynamically import Tauri OS plugin only when in Tauri environment
    const os = await import('@tauri-apps/plugin-os')
    const { getVersion, getTauriVersion } = await import('@tauri-apps/api/app')

    const [osType, osArch, appVersion, tauriVer] = await Promise.all([
      os.type(),
      os.arch(),
      getVersion().catch(() => undefined),
      getTauriVersion().catch(() => undefined),
    ])

    // Map Tauri OS type to our platform type
    const osMap: Record<string, PlatformInfo['os']> = {
      'windows': 'windows',
      'macos': 'macos',
      'linux': 'linux',
      'ios': 'ios',
      'android': 'android',
    }

    return {
      ...basicInfo,
      os: osMap[osType.toLowerCase()] || 'unknown',
      arch: osArch,
      tauriVersion: tauriVer,
      webviewVersion: appVersion,
    }
  } catch (error) {
    // If OS plugin is not available, return basic info
    console.warn('Failed to get detailed platform info:', error)
    return basicInfo
  }
}

/**
 * Reset the cached Tauri detection.
 *
 * This is primarily useful for testing purposes.
 * In normal usage, the cache improves performance by avoiding
 * repeated DOM checks.
 */
export function resetPlatformCache(): void {
  _isTauriCached = null
}

/**
 * Type guard to check if we're in Tauri for TypeScript narrowing.
 *
 * @example
 * ```ts
 * import { assertTauri } from '@/utils/platform'
 *
 * function doTauriStuff() {
 *   assertTauri() // throws if not in Tauri
 *   // TypeScript now knows we're in Tauri
 *   await invoke('command')
 * }
 * ```
 */
export function assertTauri(): void {
  if (!isTauri()) {
    throw new Error('This function requires a Tauri environment')
  }
}

/**
 * Execute a callback only in Tauri environment.
 *
 * @param callback Function to execute in Tauri environment
 * @returns The result of the callback, or undefined if not in Tauri
 *
 * @example
 * ```ts
 * import { inTauri } from '@/utils/platform'
 *
 * const result = await inTauri(async () => {
 *   const { invoke } = await import('@tauri-apps/api/core')
 *   return invoke('my_command')
 * })
 * ```
 */
export async function inTauri<T>(callback: () => T | Promise<T>): Promise<T | undefined> {
  if (!isTauri()) {
    return undefined
  }
  return callback()
}

/**
 * Execute a callback only in web browser environment.
 *
 * @param callback Function to execute in web environment
 * @returns The result of the callback, or undefined if in Tauri
 *
 * @example
 * ```ts
 * import { inWeb } from '@/utils/platform'
 *
 * const result = await inWeb(async () => {
 *   return fetch('/api/endpoint').then(r => r.json())
 * })
 * ```
 */
export async function inWeb<T>(callback: () => T | Promise<T>): Promise<T | undefined> {
  if (isTauri()) {
    return undefined
  }
  return callback()
}
