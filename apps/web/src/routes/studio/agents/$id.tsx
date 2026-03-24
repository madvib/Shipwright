import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useState, useCallback, useMemo } from 'react'
import { useAgentStore, makeAgent } from '#/features/agents/useAgentStore'
import { AgentActivityBar } from '#/features/agents/AgentActivityBar'
import { AgentStickyHeader } from '#/features/agents/AgentStickyHeader'
import { useScrollspy } from '#/features/agents/useScrollspy'
import { SkillsSection } from '#/features/agents/sections/SkillsSection'
import { McpSection } from '#/features/agents/sections/McpSection'
import { PermissionsSection } from '#/features/agents/sections/PermissionsSection'
import { ProviderSettingsSection } from '#/features/agents/sections/ProviderSettingsSection'
import { RulesSection } from '#/features/agents/sections/RulesSection'
import { HooksSection } from '#/features/agents/sections/HooksSection'
import { AddSkillDialog } from '#/features/agents/dialogs/AddSkillDialog'
import { AddMcpDialog } from '#/features/agents/dialogs/AddMcpDialog'
import { EditAgentDialog } from '#/features/agents/dialogs/EditAgentDialog'
import { PermissionsDialog } from '#/features/agents/dialogs/PermissionsDialog'
import { RuleEditorDialog } from '#/features/agents/dialogs/RuleEditorDialog'
import { HookEditorDialog } from '#/features/agents/dialogs/HookEditorDialog'
import type { ResolvedAgentProfile, ToolPermission } from '#/features/agents/types'
import type { Skill, Rule, HookConfig, ProfilePermissions } from '@ship/ui'

import { AgentDetailSkeleton } from '#/features/studio/StudioSkeleton'
import { StudioErrorBoundary } from '#/features/studio/StudioErrorBoundary'

export const Route = createFileRoute('/studio/agents/$id')({
  component: AgentDetailPage,
  pendingComponent: AgentDetailSkeleton,
  errorComponent: StudioErrorBoundary,
  ssr: false,
})

function AgentDetailPage() {
  const { id } = Route.useParams()
  const navigate = useNavigate()
  const { getAgent, updateAgent, deleteAgent } = useAgentStore()
  const profile = getAgent(id) ?? makeAgent({ profile: { id, name: id } })
  const { scrollRef, activeSection, handleSectionClick } = useScrollspy()

  const handleDelete = useCallback(() => {
    if (!confirm(`Delete "${profile.profile.name}"?`)) return
    deleteAgent(id)
    void navigate({ to: '/studio/agents', replace: true })
  }, [id, profile.profile.name, deleteAgent, navigate])

  const update = useCallback(
    (patch: Partial<ResolvedAgentProfile>) => updateAgent(id, patch),
    [id, updateAgent],
  )

  // ── Skill mutators ────────────────────────────────────────────────────
  const removeSkill = useCallback(
    (skillId: string) => update({ skills: profile.skills.filter((s) => s.id !== skillId) }),
    [update, profile.skills],
  )
  const addSkill = useCallback(
    (skill: Skill) => {
      if (profile.skills.some((s) => s.id === skill.id)) return
      update({ skills: [...profile.skills, skill] })
    },
    [update, profile.skills],
  )

  // ── MCP mutators ──────────────────────────────────────────────────────
  const removeServer = useCallback(
    (name: string) => update({ mcpServers: profile.mcpServers.filter((s) => s.name !== name) }),
    [update, profile.mcpServers],
  )
  const mcpToolStates = profile.toolPermissions ?? {}
  const setToolPermission = useCallback(
    (serverName: string, toolName: string, permission: ToolPermission) => {
      const prev = profile.toolPermissions ?? {}
      const serverTools = prev[serverName] ?? {}
      update({ toolPermissions: { ...prev, [serverName]: { ...serverTools, [toolName]: permission } } })
    },
    [update, profile.toolPermissions],
  )
  const setGroupPermission = useCallback(
    (serverName: string, toolNames: string[], permission: ToolPermission) => {
      const prev = profile.toolPermissions ?? {}
      const serverTools = { ...(prev[serverName] ?? {}) }
      for (const name of toolNames) serverTools[name] = permission
      update({ toolPermissions: { ...prev, [serverName]: serverTools } })
    },
    [update, profile.toolPermissions],
  )

  // ── Permissions mutators ──────────────────────────────────────────────
  const setPermissionPreset = useCallback(
    (preset: string) => update({ permissions: { ...profile.permissions, preset } }),
    [update, profile.permissions],
  )
  const updatePermissions = useCallback(
    (permissions: ProfilePermissions) => update({ permissions: { ...permissions, preset: 'custom' } }),
    [update],
  )

  // ── Rule mutators ─────────────────────────────────────────────────────
  const addRule = useCallback(
    (rule: Rule) => update({ rules: [...profile.rules, rule] }),
    [update, profile.rules],
  )
  const updateRule = useCallback(
    (index: number, rule: Rule) =>
      update({ rules: profile.rules.map((r, i) => (i === index ? rule : r)) }),
    [update, profile.rules],
  )
  const removeRule = useCallback(
    (index: number) => update({ rules: profile.rules.filter((_, i) => i !== index) }),
    [update, profile.rules],
  )

  // ── Hook mutators ─────────────────────────────────────────────────────
  const addHook = useCallback(
    (hook: HookConfig) => update({ hooks: [...profile.hooks, hook] }),
    [update, profile.hooks],
  )
  const updateHook = useCallback(
    (index: number, hook: HookConfig) =>
      update({ hooks: profile.hooks.map((h, i) => (i === index ? hook : h)) }),
    [update, profile.hooks],
  )
  const removeHook = useCallback(
    (index: number) => update({ hooks: profile.hooks.filter((_, i) => i !== index) }),
    [update, profile.hooks],
  )

  // ── Provider settings ─────────────────────────────────────────────────
  const providerSettings = profile.providerSettings ?? {}
  const setProviderSettings = useCallback(
    (settings: Record<string, Record<string, unknown>>) => update({ providerSettings: settings }),
    [update],
  )

  // ── Dialog state ──────────────────────────────────────────────────────
  const [skillOpen, setSkillOpen] = useState(false)
  const [mcpOpen, setMcpOpen] = useState(false)
  const [editOpen, setEditOpen] = useState(false)
  const [permsOpen, setPermsOpen] = useState(false)
  const [ruleOpen, setRuleOpen] = useState(false)
  const [ruleEdit, setRuleEdit] = useState<{ index: number; rule: Rule } | null>(null)
  const [hookOpen, setHookOpen] = useState(false)
  const [hookEdit, setHookEdit] = useState<{ index: number; hook: HookConfig } | null>(null)

  const counts = useMemo(() => ({
    skills: profile.skills.length,
    mcp: profile.mcpServers.length,
    rules: profile.rules.length,
    hooks: profile.hooks.length,
  }), [profile.skills.length, profile.mcpServers.length, profile.rules.length, profile.hooks.length])

  return (
    <>
      <div className="flex h-full min-h-0 overflow-hidden">
        <AgentActivityBar activeSection={activeSection} onSectionClick={handleSectionClick} counts={counts} />
        <div className="flex-1 min-w-0 flex flex-col overflow-hidden">
          <AgentStickyHeader profile={profile} onEdit={() => setEditOpen(true)} onDelete={handleDelete} />
          <div ref={scrollRef} className="flex-1 overflow-y-auto">
            <div id="section-skills">
              <SkillsSection skills={profile.skills} onRemove={removeSkill} onAdd={() => setSkillOpen(true)} />
            </div>
            <div id="section-mcp">
              <McpSection servers={profile.mcpServers} toolStates={mcpToolStates} onRemove={removeServer} onSetToolPermission={setToolPermission} onSetGroupPermission={setGroupPermission} onAdd={() => setMcpOpen(true)} />
            </div>
            <div id="section-permissions">
              <PermissionsSection permissions={profile.permissions ?? {}} activePreset={profile.permissions?.preset ?? 'ship-standard'} onPresetChange={setPermissionPreset} onEdit={() => setPermsOpen(true)} />
            </div>
            <div id="section-rules">
              <RulesSection rules={profile.rules} onAdd={() => { setRuleEdit(null); setRuleOpen(true) }} onEdit={(i) => { setRuleEdit({ index: i, rule: profile.rules[i] }); setRuleOpen(true) }} onRemove={removeRule} />
            </div>
            <div id="section-hooks">
              <HooksSection hooks={profile.hooks} onAdd={() => { setHookEdit(null); setHookOpen(true) }} onEdit={(i) => { setHookEdit({ index: i, hook: profile.hooks[i] }); setHookOpen(true) }} onRemove={removeHook} />
            </div>
            <div id="section-providers">
              <ProviderSettingsSection providers={['claude', 'gemini', 'codex', 'cursor', 'opencode']} providerSettings={providerSettings} onChange={setProviderSettings} />
            </div>
            <div className="h-24" />
          </div>
        </div>
      </div>

      <AddSkillDialog open={skillOpen} onOpenChange={setSkillOpen} existingIds={profile.skills.map((s) => s.id)} onAdd={addSkill} />
      <AddMcpDialog open={mcpOpen} onOpenChange={setMcpOpen} existingNames={profile.mcpServers.map((s) => s.name)} onAdd={(server) => update({ mcpServers: [...profile.mcpServers, server] })} />
      <EditAgentDialog open={editOpen} onOpenChange={setEditOpen} profile={profile} onSave={(patch) => update(patch)} />
      <PermissionsDialog open={permsOpen} onOpenChange={setPermsOpen} permissions={profile.permissions ?? {}} onSave={updatePermissions} />
      <RuleEditorDialog open={ruleOpen} onOpenChange={setRuleOpen} rule={ruleEdit?.rule ?? null} onSave={(rule) => { if (ruleEdit) updateRule(ruleEdit.index, rule); else addRule(rule) }} onDelete={ruleEdit ? () => removeRule(ruleEdit.index) : undefined} />
      <HookEditorDialog open={hookOpen} onOpenChange={setHookOpen} hook={hookEdit?.hook ?? null} onSave={(hook) => { if (hookEdit) updateHook(hookEdit.index, hook); else addHook(hook) }} onDelete={hookEdit ? () => removeHook(hookEdit.index) : undefined} />
    </>
  )
}
