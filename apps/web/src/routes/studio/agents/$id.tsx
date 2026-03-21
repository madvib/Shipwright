import { createFileRoute } from '@tanstack/react-router'
import { useState, useCallback } from 'react'
import { useAgentStore, makeAgent } from '#/features/agents/useAgentStore'
import { AgentHeader } from '#/features/agents/sections/AgentHeader'
import { SkillsSection } from '#/features/agents/sections/SkillsSection'
import { McpSection } from '#/features/agents/sections/McpSection'
import { SubagentsSection } from '#/features/agents/sections/SubagentsSection'
import { PermissionsSection } from '#/features/agents/sections/PermissionsSection'
import { ProviderSettingsSection } from '#/features/agents/sections/ProviderSettingsSection'
import { SettingsSection } from '#/features/agents/sections/SettingsSection'
import { HooksSection } from '#/features/agents/sections/HooksSection'
import { RulesSection } from '#/features/agents/sections/RulesSection'
import { AddSkillDialog } from '#/features/agents/dialogs/AddSkillDialog'
import { AddMcpDialog } from '#/features/agents/dialogs/AddMcpDialog'
import { AddSubagentDialog } from '#/features/agents/dialogs/AddSubagentDialog'
import { EditAgentDialog } from '#/features/agents/dialogs/EditAgentDialog'
import { PermissionsDialog } from '#/features/agents/dialogs/PermissionsDialog'
import { HookEditorDialog } from '#/features/agents/dialogs/HookEditorDialog'
import { RuleEditorDialog } from '#/features/agents/dialogs/RuleEditorDialog'
import type { HookConfig, AgentProfile, AgentSettings, ToolPermission } from '#/features/agents/types'
import type { Skill, Rule, Permissions } from '@ship/ui'

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
  const { getAgent, updateAgent } = useAgentStore()
  const profile = getAgent(id) ?? makeAgent({ id, name: id })

  // ── Convenience mutators (same API as old useAgentDetail) ─────────────

  const update = useCallback(
    (patch: Partial<AgentProfile>) => updateAgent(id, patch),
    [id, updateAgent],
  )

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

  const removeServer = useCallback(
    (name: string) => update({ mcpServers: profile.mcpServers.filter((s) => s.name !== name) }),
    [update, profile.mcpServers],
  )

  const removeSubagent = useCallback(
    (subId: string) => update({ subagents: profile.subagents.filter((s) => s.id !== subId) }),
    [update, profile.subagents],
  )

  const setPermissionPreset = useCallback(
    (preset: string) => update({ permissionPreset: preset }),
    [update],
  )

  const updateSettings = useCallback(
    (patch: Partial<AgentSettings>) => update({ settings: { ...profile.settings, ...patch } }),
    [update, profile.settings],
  )

  const setToolPermission = useCallback(
    (serverName: string, toolName: string, permission: ToolPermission) => {
      const serverTools = profile.mcpToolStates[serverName] ?? {}
      update({
        mcpToolStates: {
          ...profile.mcpToolStates,
          [serverName]: { ...serverTools, [toolName]: permission },
        },
      })
    },
    [update, profile.mcpToolStates],
  )

  const setGroupPermission = useCallback(
    (serverName: string, toolNames: string[], permission: ToolPermission) => {
      const serverTools = { ...(profile.mcpToolStates[serverName] ?? {}) }
      for (const name of toolNames) serverTools[name] = permission
      update({
        mcpToolStates: { ...profile.mcpToolStates, [serverName]: serverTools },
      })
    },
    [update, profile.mcpToolStates],
  )

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

  const updatePermissions = useCallback(
    (permissions: Permissions) => update({ permissions, permissionPreset: 'custom' }),
    [update],
  )

  const setMaxTurns = useCallback(
    (maxTurns: number | undefined) => update({ maxTurns }),
    [update],
  )

  const setProviderSettings = useCallback(
    (providerSettings: Record<string, Record<string, unknown>>) =>
      update({ providerSettings }),
    [update],
  )

  // ── Dialog state ──────────────────────────────────────────────────────

  const [skillOpen, setSkillOpen] = useState(false)
  const [mcpOpen, setMcpOpen] = useState(false)
  const [subagentOpen, setSubagentOpen] = useState(false)
  const [editOpen, setEditOpen] = useState(false)
  const [permsOpen, setPermsOpen] = useState(false)
  const [hookOpen, setHookOpen] = useState(false)
  const [hookEdit, setHookEdit] = useState<{ index: number; hook: HookConfig } | null>(null)
  const [ruleOpen, setRuleOpen] = useState(false)
  const [ruleEdit, setRuleEdit] = useState<{ index: number; rule: Rule } | null>(null)

  return (
    <main className="flex-1 overflow-y-auto">
      <div className="mx-auto max-w-[800px]">
        <AgentHeader profile={profile} onEdit={() => setEditOpen(true)} />

        <SkillsSection skills={profile.skills} onRemove={removeSkill} onAdd={() => setSkillOpen(true)} />

        <McpSection
          servers={profile.mcpServers}
          toolStates={profile.mcpToolStates}
          onRemove={removeServer}
          onSetToolPermission={setToolPermission}
          onSetGroupPermission={setGroupPermission}
          onAdd={() => setMcpOpen(true)}
        />

        <SubagentsSection subagents={profile.subagents} onRemove={removeSubagent} onAdd={() => setSubagentOpen(true)} />

        <PermissionsSection
          permissions={profile.permissions}
          activePreset={profile.permissionPreset}
          maxTurns={profile.maxTurns}
          onPresetChange={setPermissionPreset}
          onMaxTurnsChange={setMaxTurns}
          onEdit={() => setPermsOpen(true)}
        />

        <ProviderSettingsSection
          providers={profile.providers}
          providerSettings={profile.providerSettings ?? {}}
          onChange={setProviderSettings}
        />

        <SettingsSection settings={profile.settings} onUpdate={updateSettings} />

        <HooksSection
          hooks={profile.hooks}
          onAdd={() => { setHookEdit(null); setHookOpen(true) }}
          onEdit={(i) => { setHookEdit({ index: i, hook: profile.hooks[i] }); setHookOpen(true) }}
          onRemove={removeHook}
        />

        <RulesSection
          rules={profile.rules}
          onAdd={() => { setRuleEdit(null); setRuleOpen(true) }}
          onEdit={(i) => { setRuleEdit({ index: i, rule: profile.rules[i] }); setRuleOpen(true) }}
          onRemove={removeRule}
        />

        <div className="h-24" />
      </div>

      <AddSkillDialog
        open={skillOpen}
        onOpenChange={setSkillOpen}
        existingIds={profile.skills.map((s) => s.id)}
        onAdd={addSkill}
      />

      <AddMcpDialog
        open={mcpOpen}
        onOpenChange={setMcpOpen}
        existingNames={profile.mcpServers.map((s) => s.name)}
        onAdd={(server) => update({ mcpServers: [...profile.mcpServers, server] })}
      />

      <AddSubagentDialog
        open={subagentOpen}
        onOpenChange={setSubagentOpen}
        currentAgentId={id}
        existingIds={profile.subagents.map((s) => s.id)}
        onAdd={(ref) => update({ subagents: [...profile.subagents, ref] })}
      />

      <EditAgentDialog
        open={editOpen}
        onOpenChange={setEditOpen}
        profile={profile}
        onSave={(patch) => update(patch)}
      />

      <PermissionsDialog
        open={permsOpen}
        onOpenChange={setPermsOpen}
        permissions={profile.permissions}
        onSave={updatePermissions}
      />

      <HookEditorDialog
        open={hookOpen}
        onOpenChange={setHookOpen}
        hook={hookEdit?.hook ?? null}
        onSave={(hook) => {
          if (hookEdit) updateHook(hookEdit.index, hook)
          else addHook(hook)
        }}
        onDelete={hookEdit ? () => removeHook(hookEdit.index) : undefined}
      />

      <RuleEditorDialog
        open={ruleOpen}
        onOpenChange={setRuleOpen}
        rule={ruleEdit?.rule ?? null}
        onSave={(rule) => {
          if (ruleEdit) updateRule(ruleEdit.index, rule)
          else addRule(rule)
        }}
        onDelete={ruleEdit ? () => removeRule(ruleEdit.index) : undefined}
      />
    </main>
  )
}
