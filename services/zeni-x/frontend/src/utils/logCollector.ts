/**
 * Browser Console Log Collector
 *
 * This module intercepts console methods (log, warn, error, debug, info)
 * and sends them to the backend log aggregation service.
 *
 * Only active in web mode (when window.__TAURI__ is not available).
 */

type LogLevel = 'debug' | 'info' | 'warn' | 'error' | 'log'

interface LogEntry {
  source: 'browser'
  level: LogLevel
  message: string
}

// Store original console methods
const originalConsole = {
  log: console.log.bind(console),
  info: console.info.bind(console),
  warn: console.warn.bind(console),
  error: console.error.bind(console),
  debug: console.debug.bind(console),
}

// Queue for batching logs
let logQueue: LogEntry[] = []
let flushTimeout: ReturnType<typeof setTimeout> | null = null
const FLUSH_INTERVAL = 100 // ms
const MAX_QUEUE_SIZE = 50

// Backend API endpoint
const LOG_API_URL = 'http://127.0.0.1:12420/api/logs'

/**
 * Format console arguments into a string message
 */
function formatArgs(args: unknown[]): string {
  return args
    .map((arg) => {
      if (typeof arg === 'string') {
        return arg
      }
      if (arg instanceof Error) {
        return `${arg.name}: ${arg.message}\n${arg.stack || ''}`
      }
      try {
        return JSON.stringify(arg, null, 2)
      } catch {
        return String(arg)
      }
    })
    .join(' ')
}

/**
 * Send logs to the backend
 */
async function flushLogs(): Promise<void> {
  if (logQueue.length === 0) return

  const logsToSend = [...logQueue]
  logQueue = []

  // Send each log entry individually
  for (const log of logsToSend) {
    try {
      await fetch(LOG_API_URL, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(log),
      })
    } catch {
      // Silently ignore errors to prevent infinite loops
    }
  }
}

/**
 * Queue a log entry for sending
 */
function queueLog(level: LogLevel, args: unknown[]): void {
  const message = formatArgs(args)

  // Skip empty messages
  if (!message.trim()) return

  // Skip logs from the log collector itself
  if (message.includes(LOG_API_URL)) return

  logQueue.push({
    source: 'browser',
    level,
    message,
  })

  // Flush if queue is full
  if (logQueue.length >= MAX_QUEUE_SIZE) {
    if (flushTimeout) {
      clearTimeout(flushTimeout)
      flushTimeout = null
    }
    flushLogs()
  } else if (!flushTimeout) {
    // Schedule flush
    flushTimeout = setTimeout(() => {
      flushTimeout = null
      flushLogs()
    }, FLUSH_INTERVAL)
  }
}

/**
 * Create an intercepted console method
 */
function createInterceptor(
  level: LogLevel,
  originalMethod: (...args: unknown[]) => void
): (...args: unknown[]) => void {
  return (...args: unknown[]) => {
    // Call original method first
    originalMethod(...args)
    // Queue for sending to backend
    queueLog(level, args)
  }
}

/**
 * Initialize the log collector
 * Only activates in web mode (no Tauri)
 */
export function initLogCollector(): void {
  // Check if we're in Tauri mode
  if ((window as unknown as { __TAURI__?: unknown }).__TAURI__) {
    originalConsole.info('[LogCollector] Tauri detected, skipping initialization')
    return
  }

  // Check if already initialized
  if ((console as unknown as { __intercepted?: boolean }).__intercepted) {
    return
  }

  // Replace console methods
  console.log = createInterceptor('log', originalConsole.log)
  console.info = createInterceptor('info', originalConsole.info)
  console.warn = createInterceptor('warn', originalConsole.warn)
  console.error = createInterceptor('error', originalConsole.error)
  console.debug = createInterceptor('debug', originalConsole.debug)

  // Mark as initialized
  ;(console as unknown as { __intercepted?: boolean }).__intercepted = true

  originalConsole.info('[LogCollector] Browser console logging enabled')
}

/**
 * Restore original console methods
 */
export function restoreConsole(): void {
  console.log = originalConsole.log
  console.info = originalConsole.info
  console.warn = originalConsole.warn
  console.error = originalConsole.error
  console.debug = originalConsole.debug
  ;(console as unknown as { __intercepted?: boolean }).__intercepted = false
}

/**
 * Manually send a log to the backend
 */
export function sendLog(level: LogLevel, message: string): void {
  queueLog(level, [message])
}
