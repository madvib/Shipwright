import { describe, it, expect, beforeEach } from 'vitest'
import {
  setDraft,
  clearDraft,
  clearAllDrafts,
  getDrafts,
  hasDraft,
} from '../useAgentDrafts'

describe('draft discard', () => {
  beforeEach(() => {
    clearAllDrafts()
  })

  it('clearDraft removes a single agent draft and preserves others', () => {
    setDraft('agent-a', { model: 'model-a' })
    setDraft('agent-b', { model: 'model-b' })
    setDraft('agent-c', { model: 'model-c' })

    clearDraft('agent-b')

    expect(hasDraft('agent-a')).toBe(true)
    expect(hasDraft('agent-b')).toBe(false)
    expect(hasDraft('agent-c')).toBe(true)
    expect(Object.keys(getDrafts())).toHaveLength(2)
  })

  it('clearDraft is idempotent for non-existent draft', () => {
    setDraft('agent-a', { model: 'x' })
    clearDraft('nonexistent')
    expect(hasDraft('agent-a')).toBe(true)
    expect(Object.keys(getDrafts())).toHaveLength(1)
  })

  it('clearDraft persists removal in store', () => {
    setDraft('agent-a', { model: 'x' })
    expect(hasDraft('agent-a')).toBe(true)

    clearDraft('agent-a')
    expect(hasDraft('agent-a')).toBe(false)
    expect(getDrafts()['agent-a']).toBeUndefined()
  })

  it('after discard, re-applying a draft works', () => {
    setDraft('agent-a', { model: 'old' })
    clearDraft('agent-a')
    expect(hasDraft('agent-a')).toBe(false)

    setDraft('agent-a', { model: 'new' })
    expect(hasDraft('agent-a')).toBe(true)
    expect(getDrafts()['agent-a']).toEqual({ model: 'new' })
  })
})
