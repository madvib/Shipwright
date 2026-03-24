import { describe, it, expect, beforeEach } from 'vitest'
import {
  setDraft,
  clearDraft,
  clearAllDrafts,
  getDrafts,
  hasDraft,
} from '../useAgentDrafts'

const STORAGE_KEY = 'ship-agent-drafts-v1'

describe('useAgentDrafts store', () => {
  beforeEach(() => {
    clearAllDrafts()
    localStorage.removeItem(STORAGE_KEY)
  })

  it('starts with empty drafts', () => {
    expect(getDrafts()).toEqual({})
  })

  it('setDraft creates a draft for an agent', () => {
    setDraft('agent-1', { model: 'claude-sonnet-4' })
    expect(hasDraft('agent-1')).toBe(true)
    expect(getDrafts()['agent-1']).toEqual({ model: 'claude-sonnet-4' })
  })

  it('setDraft merges patches for the same agent', () => {
    setDraft('agent-1', { model: 'claude-sonnet-4' })
    setDraft('agent-1', { env: { API_KEY: 'test' } })
    const draft = getDrafts()['agent-1']
    expect(draft).toEqual({ model: 'claude-sonnet-4', env: { API_KEY: 'test' } })
  })

  it('clearDraft removes a specific agent draft', () => {
    setDraft('agent-1', { model: 'a' })
    setDraft('agent-2', { model: 'b' })
    clearDraft('agent-1')
    expect(hasDraft('agent-1')).toBe(false)
    expect(hasDraft('agent-2')).toBe(true)
  })

  it('clearAllDrafts removes all drafts', () => {
    setDraft('agent-1', { model: 'a' })
    setDraft('agent-2', { model: 'b' })
    clearAllDrafts()
    expect(getDrafts()).toEqual({})
  })

  it('hasDraft returns false for unknown agent', () => {
    expect(hasDraft('nonexistent')).toBe(false)
  })

  it('persists drafts to localStorage', () => {
    setDraft('agent-1', { model: 'test' })
    const raw = localStorage.getItem(STORAGE_KEY)
    expect(raw).toBeTruthy()
    const parsed = JSON.parse(raw!)
    expect(parsed.drafts['agent-1']).toEqual({ model: 'test' })
  })

  it('clearDraft persists removal to localStorage', () => {
    setDraft('agent-1', { model: 'a' })
    clearDraft('agent-1')
    const raw = localStorage.getItem(STORAGE_KEY)
    const parsed = JSON.parse(raw!)
    expect(parsed.drafts['agent-1']).toBeUndefined()
  })

  it('handles multiple agents independently', () => {
    setDraft('a', { model: 'model-a' })
    setDraft('b', { model: 'model-b' })
    setDraft('c', { model: 'model-c' })
    expect(Object.keys(getDrafts())).toEqual(['a', 'b', 'c'])
    clearDraft('b')
    expect(Object.keys(getDrafts())).toEqual(['a', 'c'])
  })
})
