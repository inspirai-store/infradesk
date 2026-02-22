/**
 * Utility functions for Zeni-X frontend
 */

// Platform detection utilities
export {
  isTauri,
  isWeb,
  getPlatformInfo,
  getPlatformInfoAsync,
  resetPlatformCache,
  assertTauri,
  inTauri,
  inWeb,
} from './platform'
export type { PlatformInfo } from './platform'
