import { describe, it, expect } from 'vitest'
import { combineSyncStatuses } from '../SyncStatus'
import type { SyncStatusValue } from '../SyncStatus'

describe('combineSyncStatuses', () => {
  it('returns idle when all statuses are idle', () => {
    expect(combineSyncStatuses('idle', 'idle')).toBe('idle')
  })

  it('returns saving when any status is saving', () => {
    expect(combineSyncStatuses('idle', 'saving')).toBe('saving')
    expect(combineSyncStatuses('saving', 'saved')).toBe('saving')
    expect(combineSyncStatuses('saving', 'error')).toBe('saving')
  })

  it('returns error when any status is error and none is saving', () => {
    expect(combineSyncStatuses('idle', 'error')).toBe('error')
    expect(combineSyncStatuses('saved', 'error')).toBe('error')
    expect(combineSyncStatuses('error', 'error')).toBe('error')
  })

  it('returns saved when any status is saved and none is saving or error', () => {
    expect(combineSyncStatuses('idle', 'saved')).toBe('saved')
    expect(combineSyncStatuses('saved', 'saved')).toBe('saved')
  })

  it('handles single status', () => {
    const cases: SyncStatusValue[] = ['idle', 'saving', 'saved', 'error']
    for (const s of cases) {
      expect(combineSyncStatuses(s)).toBe(s)
    }
  })

  it('handles three or more statuses', () => {
    expect(combineSyncStatuses('idle', 'idle', 'saving')).toBe('saving')
    expect(combineSyncStatuses('saved', 'idle', 'error')).toBe('error')
    expect(combineSyncStatuses('saved', 'idle', 'idle')).toBe('saved')
  })
})
