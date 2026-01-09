import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import {
  isTauri,
  isWeb,
  getPlatformInfo,
  resetPlatformCache,
  assertTauri,
  inTauri,
  inWeb,
} from './platform'

describe('Platform Detection Utility', () => {
  // Store original window properties
  const originalWindow = global.window

  beforeEach(() => {
    // Reset platform cache before each test
    resetPlatformCache()
  })

  afterEach(() => {
    // Restore window after each test
    global.window = originalWindow
    resetPlatformCache()
    vi.restoreAllMocks()
  })

  describe('isTauri()', () => {
    it('returns false when window is undefined (Node.js environment)', () => {
      // @ts-expect-error - Testing undefined window
      global.window = undefined
      expect(isTauri()).toBe(false)
    })

    it('returns false when __TAURI_INTERNALS__ is not present', () => {
      // Simulate a regular browser window
      global.window = {} as Window & typeof globalThis
      expect(isTauri()).toBe(false)
    })

    it('returns true when __TAURI_INTERNALS__ is present (Tauri 2.x)', () => {
      // Simulate Tauri 2.x environment
      global.window = {
        __TAURI_INTERNALS__: { invoke: vi.fn() },
      } as unknown as Window & typeof globalThis
      expect(isTauri()).toBe(true)
    })

    it('returns true when __TAURI_IPC__ is present (Tauri 1.x compatibility)', () => {
      // Simulate Tauri 1.x environment
      global.window = {
        __TAURI_IPC__: vi.fn(),
      } as unknown as Window & typeof globalThis
      expect(isTauri()).toBe(true)
    })

    it('returns true when __TAURI_METADATA__ is present', () => {
      // Simulate Tauri metadata presence
      global.window = {
        __TAURI_METADATA__: { platform: 'macos' },
      } as unknown as Window & typeof globalThis
      expect(isTauri()).toBe(true)
    })

    it('caches the result for performance', () => {
      // First call with Tauri environment
      global.window = {
        __TAURI_INTERNALS__: { invoke: vi.fn() },
      } as unknown as Window & typeof globalThis
      expect(isTauri()).toBe(true)

      // Change window but cached value should persist
      global.window = {} as Window & typeof globalThis
      expect(isTauri()).toBe(true) // Still returns cached true
    })

    it('can reset cache for testing', () => {
      // Set up Tauri environment
      global.window = {
        __TAURI_INTERNALS__: { invoke: vi.fn() },
      } as unknown as Window & typeof globalThis
      expect(isTauri()).toBe(true)

      // Reset cache and change environment
      resetPlatformCache()
      global.window = {} as Window & typeof globalThis
      expect(isTauri()).toBe(false) // Now returns false
    })
  })

  describe('isWeb()', () => {
    it('returns true when not in Tauri (regular browser)', () => {
      global.window = {} as Window & typeof globalThis
      expect(isWeb()).toBe(true)
    })

    it('returns false when in Tauri', () => {
      global.window = {
        __TAURI_INTERNALS__: { invoke: vi.fn() },
      } as unknown as Window & typeof globalThis
      expect(isWeb()).toBe(false)
    })

    it('is the inverse of isTauri()', () => {
      global.window = {} as Window & typeof globalThis
      expect(isWeb()).toBe(!isTauri())

      resetPlatformCache()
      global.window = {
        __TAURI_INTERNALS__: { invoke: vi.fn() },
      } as unknown as Window & typeof globalThis
      expect(isWeb()).toBe(!isTauri())
    })
  })

  describe('getPlatformInfo()', () => {
    it('returns correct info for web environment', () => {
      global.window = {} as Window & typeof globalThis
      const info = getPlatformInfo()

      expect(info.isTauri).toBe(false)
      expect(info.isWeb).toBe(true)
      expect(info.os).toBeUndefined()
      expect(info.arch).toBeUndefined()
      expect(info.tauriVersion).toBeUndefined()
    })

    it('returns correct info for Tauri environment', () => {
      global.window = {
        __TAURI_INTERNALS__: { invoke: vi.fn() },
      } as unknown as Window & typeof globalThis
      const info = getPlatformInfo()

      expect(info.isTauri).toBe(true)
      expect(info.isWeb).toBe(false)
    })
  })

  describe('assertTauri()', () => {
    it('throws error when not in Tauri', () => {
      global.window = {} as Window & typeof globalThis
      expect(() => assertTauri()).toThrowError('This function requires a Tauri environment')
    })

    it('does not throw when in Tauri', () => {
      global.window = {
        __TAURI_INTERNALS__: { invoke: vi.fn() },
      } as unknown as Window & typeof globalThis
      expect(() => assertTauri()).not.toThrow()
    })
  })

  describe('inTauri()', () => {
    it('executes callback in Tauri environment', async () => {
      global.window = {
        __TAURI_INTERNALS__: { invoke: vi.fn() },
      } as unknown as Window & typeof globalThis

      const callback = vi.fn().mockResolvedValue('tauri-result')
      const result = await inTauri(callback)

      expect(callback).toHaveBeenCalled()
      expect(result).toBe('tauri-result')
    })

    it('returns undefined and does not execute callback in web environment', async () => {
      global.window = {} as Window & typeof globalThis

      const callback = vi.fn().mockResolvedValue('tauri-result')
      const result = await inTauri(callback)

      expect(callback).not.toHaveBeenCalled()
      expect(result).toBeUndefined()
    })

    it('works with sync callbacks', async () => {
      global.window = {
        __TAURI_INTERNALS__: { invoke: vi.fn() },
      } as unknown as Window & typeof globalThis

      const result = await inTauri(() => 'sync-result')
      expect(result).toBe('sync-result')
    })
  })

  describe('inWeb()', () => {
    it('executes callback in web environment', async () => {
      global.window = {} as Window & typeof globalThis

      const callback = vi.fn().mockResolvedValue('web-result')
      const result = await inWeb(callback)

      expect(callback).toHaveBeenCalled()
      expect(result).toBe('web-result')
    })

    it('returns undefined and does not execute callback in Tauri environment', async () => {
      global.window = {
        __TAURI_INTERNALS__: { invoke: vi.fn() },
      } as unknown as Window & typeof globalThis

      const callback = vi.fn().mockResolvedValue('web-result')
      const result = await inWeb(callback)

      expect(callback).not.toHaveBeenCalled()
      expect(result).toBeUndefined()
    })

    it('works with sync callbacks', async () => {
      global.window = {} as Window & typeof globalThis

      const result = await inWeb(() => 'sync-result')
      expect(result).toBe('sync-result')
    })
  })

  describe('resetPlatformCache()', () => {
    it('allows re-detection after environment change', () => {
      // Start in web
      global.window = {} as Window & typeof globalThis
      expect(isTauri()).toBe(false)

      // Switch to Tauri (but cached)
      global.window = {
        __TAURI_INTERNALS__: { invoke: vi.fn() },
      } as unknown as Window & typeof globalThis
      expect(isTauri()).toBe(false) // Still cached as false

      // Reset and re-detect
      resetPlatformCache()
      expect(isTauri()).toBe(true) // Now correctly detects Tauri
    })
  })
})
