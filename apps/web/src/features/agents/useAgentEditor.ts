// Two-layer editor hook: working copy (React state) + persisted layer (localStorage).
// Mutations update the working copy only. Explicit save() flushes to localStorage.

import { useCallback, useEffect, useReducer, useRef } from 'react'
import { useAgentStore, makeAgent } from './useAgentStore'
import { usePanicSave } from './PanicSaveContext'
import type { ResolvedAgentProfile, AgentDraftMeta, AgentStatus } from './types'
import type { Skill, Rule, HookConfig, ProfilePermissions } from '@ship/ui'

// ── Reducer ──────────────────────────────────────────────────────────────────

interface EditorState {
  agent: ResolvedAgentProfile
  meta: AgentDraftMeta
  /** Snapshot at last save/mount for dirty comparison */
  _persistedJson: string
}

type EditorAction =
  | { type: 'set_agent'; agent: ResolvedAgentProfile }
  | { type: 'update_profile'; patch: Partial<ResolvedAgentProfile['profile']> }
  | { type: 'set_skills'; skills: Skill[] }
  | { type: 'set_mcp_servers'; servers: ResolvedAgentProfile['mcpServers'] }
  | { type: 'set_permissions'; permissions: ProfilePermissions }
  | { type: 'set_rules'; rules: Rule[] }
  | { type: 'set_hooks'; hooks: HookConfig[] }
  | { type: 'set_model'; model: string }
  | { type: 'set_provider_settings'; provider: string; settings: Record<string, unknown> }
  | { type: 'saved'; json: string }
  | { type: 'revert'; agent: ResolvedAgentProfile; json: string; status: AgentStatus }

function computeDirty(agent: ResolvedAgentProfile, persistedJson: string): boolean {
  return JSON.stringify(agent) !== persistedJson
}

function editorReducer(state: EditorState, action: EditorAction): EditorState {
  switch (action.type) {
    case 'set_agent': {
      const agent = action.agent
      return {
        ...state,
        agent,
        meta: { ...state.meta, isDirty: computeDirty(agent, state._persistedJson) },
      }
    }
    case 'update_profile': {
      const agent = { ...state.agent, profile: { ...state.agent.profile, ...action.patch } }
      return {
        ...state,
        agent,
        meta: { ...state.meta, isDirty: computeDirty(agent, state._persistedJson) },
      }
    }
    case 'set_skills': {
      const agent = { ...state.agent, skills: action.skills }
      return {
        ...state,
        agent,
        meta: { ...state.meta, isDirty: computeDirty(agent, state._persistedJson) },
      }
    }
    case 'set_mcp_servers': {
      const agent = { ...state.agent, mcpServers: action.servers }
      return {
        ...state,
        agent,
        meta: { ...state.meta, isDirty: computeDirty(agent, state._persistedJson) },
      }
    }
    case 'set_permissions': {
      const agent = { ...state.agent, permissions: action.permissions }
      return {
        ...state,
        agent,
        meta: { ...state.meta, isDirty: computeDirty(agent, state._persistedJson) },
      }
    }
    case 'set_rules': {
      const agent = { ...state.agent, rules: action.rules }
      return {
        ...state,
        agent,
        meta: { ...state.meta, isDirty: computeDirty(agent, state._persistedJson) },
      }
    }
    case 'set_hooks': {
      const agent = { ...state.agent, hooks: action.hooks }
      return {
        ...state,
        agent,
        meta: { ...state.meta, isDirty: computeDirty(agent, state._persistedJson) },
      }
    }
    case 'set_model': {
      const ps = state.agent.providerSettings ?? {}
      // model lives at top level on the working copy — store it as a virtual field
      const agent = { ...state.agent, _model: action.model }
      return {
        ...state,
        agent: agent as ResolvedAgentProfile,
        meta: { ...state.meta, isDirty: computeDirty(agent as ResolvedAgentProfile, state._persistedJson) },
      }
    }
    case 'set_provider_settings': {
      const prev = state.agent.providerSettings ?? {}
      const agent = {
        ...state.agent,
        providerSettings: { ...prev, [action.provider]: action.settings },
      }
      return {
        ...state,
        agent,
        meta: { ...state.meta, isDirty: computeDirty(agent, state._persistedJson) },
      }
    }
    case 'saved':
      return {
        ...state,
        _persistedJson: action.json,
        meta: { ...state.meta, isDirty: false, status: 'draft' },
      }
    case 'revert':
      return {
        agent: action.agent,
        _persistedJson: action.json,
        meta: { isDirty: false, status: action.status },
      }
  }
}

// ── Hook ─────────────────────────────────────────────────────────────────────

export function useAgentEditor(agentId: string) {
  const store = useAgentStore()
  const persisted = store.getAgent(agentId)
  const isPersisted = store.isPersistedAgent(agentId)

  const initial = persisted ?? makeAgent({ profile: { id: agentId, name: agentId } })
  const initialJson = JSON.stringify(initial)
  const initialStatus: AgentStatus = isPersisted ? 'draft' : 'unsaved'

  const [state, dispatch] = useReducer(editorReducer, {
    agent: initial,
    meta: { isDirty: false, status: initialStatus },
    _persistedJson: initialJson,
  })

  // Sync if persisted version changes externally (cross-tab)
  const lastPersistedRef = useRef(initialJson)
  useEffect(() => {
    if (!persisted) return
    const json = JSON.stringify(persisted)
    if (json !== lastPersistedRef.current && !state.meta.isDirty) {
      lastPersistedRef.current = json
      dispatch({ type: 'revert', agent: persisted, json, status: 'draft' })
    }
  }, [persisted, state.meta.isDirty])

  // ── Granular mutators ───────────────────────────────────────────────────

  const updateProfile = useCallback(
    (patch: Partial<ResolvedAgentProfile['profile']>) =>
      dispatch({ type: 'update_profile', patch }),
    [],
  )

  const addSkill = useCallback(
    (skill: Skill) =>
      dispatch({ type: 'set_skills', skills: [...state.agent.skills, skill] }),
    [state.agent.skills],
  )

  const removeSkill = useCallback(
    (skillId: string) =>
      dispatch({ type: 'set_skills', skills: state.agent.skills.filter((s) => s.id !== skillId) }),
    [state.agent.skills],
  )

  const addMcpServer = useCallback(
    (server: ResolvedAgentProfile['mcpServers'][0]) =>
      dispatch({ type: 'set_mcp_servers', servers: [...state.agent.mcpServers, server] }),
    [state.agent.mcpServers],
  )

  const removeMcpServer = useCallback(
    (name: string) =>
      dispatch({ type: 'set_mcp_servers', servers: state.agent.mcpServers.filter((s) => s.name !== name) }),
    [state.agent.mcpServers],
  )

  const setPermissions = useCallback(
    (permissions: ProfilePermissions) =>
      dispatch({ type: 'set_permissions', permissions }),
    [],
  )

  const addRule = useCallback(
    (rule: Rule) =>
      dispatch({ type: 'set_rules', rules: [...state.agent.rules, rule] }),
    [state.agent.rules],
  )

  const updateRule = useCallback(
    (index: number, rule: Rule) =>
      dispatch({ type: 'set_rules', rules: state.agent.rules.map((r, i) => i === index ? rule : r) }),
    [state.agent.rules],
  )

  const removeRule = useCallback(
    (index: number) =>
      dispatch({ type: 'set_rules', rules: state.agent.rules.filter((_, i) => i !== index) }),
    [state.agent.rules],
  )

  const setHooks = useCallback(
    (hooks: HookConfig[]) =>
      dispatch({ type: 'set_hooks', hooks }),
    [],
  )

  const setModel = useCallback(
    (model: string) =>
      dispatch({ type: 'set_model', model }),
    [],
  )

  const setProviderSettings = useCallback(
    (provider: string, settings: Record<string, unknown>) =>
      dispatch({ type: 'set_provider_settings', provider, settings }),
    [],
  )

  const setToolPermission = useCallback(
    (serverName: string, toolName: string, permission: import('./types').ToolPermission) => {
      const prev = state.agent.toolPermissions ?? {}
      const serverTools = prev[serverName] ?? {}
      dispatch({
        type: 'set_agent',
        agent: {
          ...state.agent,
          toolPermissions: { ...prev, [serverName]: { ...serverTools, [toolName]: permission } },
        },
      })
    },
    [state.agent],
  )

  const setGroupPermission = useCallback(
    (serverName: string, toolNames: string[], permission: import('./types').ToolPermission) => {
      const prev = state.agent.toolPermissions ?? {}
      const serverTools = { ...(prev[serverName] ?? {}) }
      for (const name of toolNames) serverTools[name] = permission
      dispatch({
        type: 'set_agent',
        agent: {
          ...state.agent,
          toolPermissions: { ...prev, [serverName]: serverTools },
        },
      })
    },
    [state.agent],
  )

  // ── Lifecycle ───────────────────────────────────────────────────────────

  const save = useCallback(() => {
    store.saveDraft(agentId, state.agent)
    dispatch({ type: 'saved', json: JSON.stringify(state.agent) })
  }, [store, agentId, state.agent])

  const revert = useCallback(() => {
    const current = store.getAgent(agentId)
    if (current) {
      dispatch({ type: 'revert', agent: current, json: JSON.stringify(current), status: 'draft' })
    }
  }, [store, agentId])

  const panicSave = useCallback(() => {
    // Synchronous — for beforeunload / error boundary
    if (state.meta.isDirty) {
      store.saveDraft(agentId, state.agent)
    }
  }, [store, agentId, state.agent, state.meta.isDirty])

  // Register with PanicSaveContext for error boundary integration
  const panicCtx = usePanicSave()
  useEffect(() => panicCtx.register(panicSave), [panicCtx, panicSave])

  return {
    agent: state.agent,
    meta: state.meta,
    // Mutators
    updateProfile,
    addSkill,
    removeSkill,
    addMcpServer,
    removeMcpServer,
    setPermissions,
    addRule,
    updateRule,
    removeRule,
    setHooks,
    setModel,
    setProviderSettings,
    setToolPermission,
    setGroupPermission,
    // Lifecycle
    save,
    revert,
    panicSave,
  }
}
