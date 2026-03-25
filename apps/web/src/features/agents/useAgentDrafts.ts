// Draft overlay store: holds unsaved edits keyed by agent ID.
// Only stores deltas -- not full agent objects.
// Persists to IndexedDB for offline resilience (no 5 MB limit).

import { useSyncExternalStore, useCallback } from 'react'
import { idbGet, idbSet, migrateFromLocalStorage } from '#/lib/idb-cache'
import type { ResolvedAgentProfile } from './types'

const STORAGE_KEY = 'ship-agent-drafts-v1'

type AgentPatch = Partial<ResolvedAgentProfile>

interface DraftState {
  drafts: Record<string, AgentPatch>
}

// ── Store singleton ──────────────────────────────────────────────────────────

let state: DraftState = { drafts: {} }
const listeners = new Set<() => void>()

function emit() {
  for (const fn of listeners) fn()
}

// Async init: migrate from localStorage then load from IDB
function initFromIdb() {
  migrateFromLocalStorage<DraftState>(STORAGE_KEY)
    .then(async (migrated) => {
      if (migrated?.drafts) {
        state = migrated
        emit()
        return
      }
      const data = await idbGet<DraftState>(STORAGE_KEY)
      if (data?.drafts) {
        state = data
        emit()
      }
    })
    .catch(() => {})
}

if (typeof window !== 'undefined') initFromIdb()

function persistToIdb() {
  idbSet(STORAGE_KEY, state).catch(() => {})
}

// ── Public mutations ─────────────────────────────────────────────────────────

export function setDraft(agentId: string, patch: AgentPatch) {
  const existing = state.drafts[agentId] ?? {}
  state = { drafts: { ...state.drafts, [agentId]: { ...existing, ...patch } } }
  persistToIdb()
  emit()
}

export function clearDraft(agentId: string) {
  const { [agentId]: _, ...rest } = state.drafts
  state = { drafts: rest }
  persistToIdb()
  emit()
}

export function clearAllDrafts() {
  state = { drafts: {} }
  persistToIdb()
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
