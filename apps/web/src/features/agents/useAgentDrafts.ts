// Draft overlay store: holds unsaved edits keyed by agent ID.
// Only stores deltas -- not full agent objects.
// Persists to localStorage for offline resilience.

import { useSyncExternalStore, useCallback } from 'react'
import type { ResolvedAgentProfile } from './types'

const STORAGE_KEY = 'ship-agent-drafts-v1'

type AgentPatch = Partial<ResolvedAgentProfile>

interface DraftState {
  drafts: Record<string, AgentPatch>
}

// ── Store singleton ──────────────────────────────────────────────────────────

let state: DraftState = loadFromStorage()
const listeners = new Set<() => void>()

function emit() {
  for (const fn of listeners) fn()
}

function loadFromStorage(): DraftState {
  try {
    const raw = typeof window !== 'undefined'
      ? window.localStorage.getItem(STORAGE_KEY)
      : null
    if (!raw) return { drafts: {} }
    const parsed = JSON.parse(raw) as DraftState
    if (!parsed.drafts || typeof parsed.drafts !== 'object') return { drafts: {} }
    return parsed
  } catch {
    return { drafts: {} }
  }
}

function persistToStorage() {
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(state))
  } catch { /* storage full or unavailable */ }
}

// ── Public mutations ─────────────────────────────────────────────────────────

export function setDraft(agentId: string, patch: AgentPatch) {
  const existing = state.drafts[agentId] ?? {}
  state = { drafts: { ...state.drafts, [agentId]: { ...existing, ...patch } } }
  persistToStorage()
  emit()
}

export function clearDraft(agentId: string) {
  const { [agentId]: _, ...rest } = state.drafts
  state = { drafts: rest }
  persistToStorage()
  emit()
}

export function clearAllDrafts() {
  state = { drafts: {} }
  persistToStorage()
  emit()
}

export function getDrafts(): Record<string, AgentPatch> {
  return state.drafts
}

export function hasDraft(agentId: string): boolean {
  return agentId in state.drafts
}

// ── React hook ───────────────────────────────────────────────────────────────

function subscribe(cb: () => void) {
  listeners.add(cb)
  return () => { listeners.delete(cb) }
}

function getSnapshot(): DraftState {
  return state
}

function getServerSnapshot(): DraftState {
  return { drafts: {} }
}

export function useAgentDrafts() {
  const current = useSyncExternalStore(subscribe, getSnapshot, getServerSnapshot)

  return {
    drafts: current.drafts,
    setDraft: useCallback((agentId: string, patch: AgentPatch) => setDraft(agentId, patch), []),
    clearDraft: useCallback((agentId: string) => clearDraft(agentId), []),
    clearAllDrafts: useCallback(() => clearAllDrafts(), []),
    hasDraft: useCallback((agentId: string) => agentId in current.drafts, [current.drafts]),
  }
}
