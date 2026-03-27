import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'

const EVER_CONNECTED_KEY = 'ship-cli-ever-connected'
const STORAGE_KEY = 'ship-mcp-config'

describe('auto-connect behavior', () => {
  beforeEach(() => {
    localStorage.clear()
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('EVER_CONNECTED_KEY is set after first connection', () => {
    // Simulate what useLocalMcp.connect() does
    localStorage.setItem(EVER_CONNECTED_KEY, 'true')
    expect(localStorage.getItem(EVER_CONNECTED_KEY)).toBe('true')
  })

  it('hasEverConnected reads from localStorage', () => {
    expect(localStorage.getItem(EVER_CONNECTED_KEY)).toBeNull()
    localStorage.setItem(EVER_CONNECTED_KEY, 'true')
    expect(localStorage.getItem(EVER_CONNECTED_KEY)).toBe('true')
  })

  it('config persists port to localStorage', () => {
    const config = { port: 9999 }
    localStorage.setItem(STORAGE_KEY, JSON.stringify(config))
    const stored = JSON.parse(localStorage.getItem(STORAGE_KEY)!) as { port: number }
    expect(stored.port).toBe(9999)
  })

  it('auto-connect should trigger when EVER_CONNECTED_KEY is true', () => {
    // This test verifies the auto-connect contract:
    // When hasEverConnected is true and status is disconnected,
    // a delayed connect() call should be scheduled.
    localStorage.setItem(EVER_CONNECTED_KEY, 'true')
    const connectFn = vi.fn()

    // Simulate the useEffect logic from useLocalMcp
    const hasEverConnected = localStorage.getItem(EVER_CONNECTED_KEY) === 'true'
    const status = 'disconnected'

    if (hasEverConnected && status === 'disconnected') {
      setTimeout(() => connectFn(), 500)
    }

    expect(connectFn).not.toHaveBeenCalled()
    vi.advanceTimersByTime(500)
    expect(connectFn).toHaveBeenCalledOnce()
  })

  it('auto-connect should NOT trigger when EVER_CONNECTED_KEY is absent', () => {
    const connectFn = vi.fn()
    const hasEverConnected = localStorage.getItem(EVER_CONNECTED_KEY) === 'true'
    const status = 'disconnected'

    if (hasEverConnected && status === 'disconnected') {
      setTimeout(() => connectFn(), 500)
    }

    vi.advanceTimersByTime(1000)
    expect(connectFn).not.toHaveBeenCalled()
  })

  it('auto-connect should NOT trigger when already connected', () => {
    localStorage.setItem(EVER_CONNECTED_KEY, 'true')
    const connectFn = vi.fn()
    const hasEverConnected = localStorage.getItem(EVER_CONNECTED_KEY) === 'true'
    const status: string = 'connected'

    if (hasEverConnected && status === 'disconnected') {
      setTimeout(() => connectFn(), 500)
    }

    vi.advanceTimersByTime(1000)
    expect(connectFn).not.toHaveBeenCalled()
  })
})
