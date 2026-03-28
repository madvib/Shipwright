import { createFileRoute, useNavigate, Link } from '@tanstack/react-router'
import { useState, useCallback, useMemo } from 'react'
import { ArrowLeft, SearchX } from 'lucide-react'
import { useAgents } from '#/features/agents/useAgents'
import { useAgentDrafts } from '#/features/agents/useAgentDrafts'
import { AgentActivityBar } from '#/features/agents/AgentActivityBar'
import { AgentStickyHeader } from '#/features/agents/AgentStickyHeader'
import { useScrollspy } from '#/features/agents/useScrollspy'
import { SkillsSection } from '#/features/agents/sections/SkillsSection'
import { McpSection } from '#/features/agents/sections/McpSection'
import { PermissionsSection } from '#/features/agents/sections/PermissionsSection'
import { ProvidersSection } from '#/features/agents/sections/ProvidersSection'
import { RulesSection } from '#/features/agents/sections/RulesSection'
import { AddSkillDialog } from '#/features/agents/dialogs/AddSkillDialog'
import { AddMcpDialog } from '#/features/agents/dialogs/AddMcpDialog'
import { EditAgentDialog } from '#/features/agents/dialogs/EditAgentDialog'
import { PermissionsDialog } from '#/features/agents/dialogs/PermissionsDialog'
import { RuleEditorDialog } from '#/features/agents/dialogs/RuleEditorDialog'
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

function AgentNotFound({ id }: { id: string }) {
  return (
    <div className="flex flex-col items-center justify-center py-24 text-center">
      <div className="flex size-12 items-center justify-center rounded-xl border border-border bg-muted/40 mb-4">
        <SearchX className="size-5 text-muted-foreground" />
      </div>
      <p className="text-sm font-medium text-foreground">Agent not found</p>
      <p className="mt-1 text-xs text-muted-foreground max-w-xs">
        No agent with ID "{id}" exists in this project.
      </p>
      <Link
        to="/studio/agents"
        className="mt-4 inline-flex items-center gap-1.5 rounded-lg border border-border px-4 py-2 text-xs font-medium text-muted-foreground hover:text-foreground hover:border-primary/30 transition no-underline"
      >
        <ArrowLeft className="size-3" />
        Back to agents
      </Link>
    </div>
  )
}

function AgentDetailPage() {
  const { id } = Route.useParams()
  const navigate = useNavigate()
  const { getAgent, isLoading } = useAgents()
  const { setDraft, hasDraft, clearDraft } = useAgentDrafts()
  const profile = getAgent(id)
  const { scrollRef, activeSection, handleSectionClick } = useScrollspy()
  const isDraft = hasDraft(id)

  if (isLoading && !profile) {
    return <AgentDetailSkeleton />
  }

  if (!profile) {
    return <AgentNotFound id={id} />
  }

  // -- Delete ---------------------------------------------------------------
  const handleDelete = () => {
    if (!confirm(`Delete "${profile.profile.name}"?`)) return
    void navigate({ to: '/studio/agents', replace: true })
  }

  const update = (patch: Partial<ResolvedAgentProfile>) => setDraft(id, patch)
  const handleDiscard = () => clearDraft(id)

  return (
    <AgentDetailView
      profile={profile}
      isDraft={isDraft}
      activeSection={activeSection}
      scrollRef={scrollRef}
      onSectionClick={handleSectionClick}
      onUpdate={update}
      onDelete={handleDelete}
      onDiscard={handleDiscard}
    />
  )
}

// -- Inner view (split to keep AgentDetailPage under 300 lines) -------------

interface ViewProps {
  profile: ResolvedAgentProfile
  isDraft: boolean
  activeSection: string
  scrollRef: React.RefObject<HTMLDivElement | null>
  onSectionClick: (section: string) => void
  onUpdate: (patch: Partial<ResolvedAgentProfile>) => void
  onDelete: () => void
  onDiscard: () => void
}

function AgentDetailView({ profile, isDraft, activeSection, scrollRef, onSectionClick, onUpdate, onDelete, onDiscard }: ViewProps) {
  const removeSkill = useCallback(
    (skillId: string) => onUpdate({ skills: profile.skills.filter((s) => s.id !== skillId) }),
    [onUpdate, profile.skills],
  )
  const addSkill = useCallback(
    (skill: Skill) => {
      if (profile.skills.some((s) => s.id === skill.id)) return
      onUpdate({ skills: [...profile.skills, skill] })
    },
    [onUpdate, profile.skills],
  )
  const removeServer = useCallback(
    (name: string) => onUpdate({ mcpServers: profile.mcpServers.filter((s) => s.name !== name) }),
    [onUpdate, profile.mcpServers],
  )
  const mcpToolStates = profile.toolPermissions ?? {}
  const setToolPermission = useCallback(
    (serverName: string, toolName: string, permission: ToolPermission) => {
      const prev = profile.toolPermissions ?? {}
      const serverTools = prev[serverName] ?? {}
      onUpdate({ toolPermissions: { ...prev, [serverName]: { ...serverTools, [toolName]: permission } } })
    },
    [onUpdate, profile.toolPermissions],
  )
  const setGroupPermission = useCallback(
    (serverName: string, toolNames: string[], permission: ToolPermission) => {
      const prev = profile.toolPermissions ?? {}
      const serverTools = { ...(prev[serverName] ?? {}) }
      for (const name of toolNames) serverTools[name] = permission
      onUpdate({ toolPermissions: { ...prev, [serverName]: serverTools } })
    },
    [onUpdate, profile.toolPermissions],
  )
  const setPermissionPreset = useCallback(
    (preset: string) => onUpdate({ permissions: { ...profile.permissions, preset } }),
    [onUpdate, profile.permissions],
  )
  const updatePermissions = useCallback(
    (permissions: ProfilePermissions) => onUpdate({ permissions: { ...permissions, preset: 'custom' } }),
    [onUpdate],
  )
  const addRule = useCallback((rule: Rule) => onUpdate({ rules: [...profile.rules, rule] }), [onUpdate, profile.rules])
  const updateRule = useCallback(
    (index: number, rule: Rule) => onUpdate({ rules: profile.rules.map((r, i) => (i === index ? rule : r)) }),
    [onUpdate, profile.rules],
  )
  const removeRule = useCallback((index: number) => onUpdate({ rules: profile.rules.filter((_, i) => i !== index) }), [onUpdate, profile.rules])
  const setModel = useCallback((model: string | null) => onUpdate({ model }), [onUpdate])
  const setEnv = useCallback((env: Record<string, string>) => onUpdate({ env }), [onUpdate])
  const setAvailableModels = useCallback((availableModels: string[]) => onUpdate({ availableModels }), [onUpdate])
  const setAgentLimits = useCallback(
    (agentLimits: Record<string, unknown>) => onUpdate({ agentLimits: agentLimits as { max_turns?: number; max_cost_per_session?: number } }),
    [onUpdate],
  )
  const setHooks = useCallback((hooks: HookConfig[]) => onUpdate({ hooks }), [onUpdate])
  const setProviderSettings = useCallback(
    (settings: Record<string, Record<string, unknown>>) => onUpdate({ providerSettings: settings }),
    [onUpdate],
  )

  const [skillOpen, setSkillOpen] = useState(false)
  const [mcpOpen, setMcpOpen] = useState(false)
  const [editOpen, setEditOpen] = useState(false)
  const [permsOpen, setPermsOpen] = useState(false)
  const [ruleOpen, setRuleOpen] = useState(false)
  const [ruleEdit, setRuleEdit] = useState<{ index: number; rule: Rule } | null>(null)

  // -- Counts for activity bar ----------------------------------------------
  const counts = useMemo(() => ({
    skills: profile.skills.length,
    mcp: profile.mcpServers.length,
    rules: profile.rules.length,
    providers: profile.profile.providers?.length ?? 0,
  }), [profile.skills.length, profile.mcpServers.length, profile.rules.length, profile.profile.providers?.length])

  return (
    <>
      <div className="flex h-full min-h-0 overflow-hidden">
        <AgentActivityBar activeSection={activeSection} onSectionClick={onSectionClick} counts={counts} />
        <div className="flex-1 min-w-0 flex flex-col overflow-hidden">
          <AgentStickyHeader profile={profile} onEdit={() => setEditOpen(true)} onDelete={onDelete} onDiscard={isDraft ? onDiscard : undefined} isDraft={isDraft} />
          <div ref={scrollRef} className="flex-1 overflow-y-auto">
            <div id="section-skills">
              <SkillsSection skills={profile.skills} onRemove={removeSkill} onAdd={() => setSkillOpen(true)} />
            </div>
            <div id="section-mcp">
              <McpSection servers={profile.mcpServers} toolStates={mcpToolStates} onRemove={removeServer} onSetToolPermission={setToolPermission} onSetGroupPermission={setGroupPermission} onAdd={() => setMcpOpen(true)} />
            </div>
            <div id="section-permissions">
              <PermissionsSection
                permissions={profile.permissions ?? {}}
                activePreset={profile.permissions?.preset ?? 'ship-standard'}
                onPresetChange={setPermissionPreset}
                onEdit={() => setPermsOpen(true)}
              />
            </div>
            <div id="section-rules">
              <RulesSection
                rules={profile.rules}
                onAdd={() => { setRuleEdit(null); setRuleOpen(true) }}
                onEdit={(i) => { setRuleEdit({ index: i, rule: profile.rules[i] }); setRuleOpen(true) }}
                onRemove={removeRule}
              />
            </div>
            <div id="section-providers">
              <ProvidersSection providers={profile.profile.providers ?? []} model={profile.model} env={profile.env} availableModels={profile.availableModels} agentLimits={profile.agentLimits} hooks={profile.hooks} providerSettings={profile.providerSettings ?? {}} onChangeModel={setModel} onChangeEnv={setEnv} onChangeAvailableModels={setAvailableModels} onChangeAgentLimits={setAgentLimits} onChangeHooks={setHooks} onChangeProviderSettings={setProviderSettings} />
            </div>
            <div className="h-24" />
          </div>
        </div>
      </div>
      <AddSkillDialog open={skillOpen} onOpenChange={setSkillOpen} existingIds={profile.skills.map((s) => s.id)} onAdd={addSkill} />
      <AddMcpDialog open={mcpOpen} onOpenChange={setMcpOpen} existingNames={profile.mcpServers.map((s) => s.name)} onAdd={(server) => onUpdate({ mcpServers: [...profile.mcpServers, server] })} />
      <EditAgentDialog open={editOpen} onOpenChange={setEditOpen} profile={profile} onSave={(patch) => onUpdate(patch)} />
      <PermissionsDialog open={permsOpen} onOpenChange={setPermsOpen} permissions={profile.permissions ?? {}} onSave={updatePermissions} />
      <RuleEditorDialog open={ruleOpen} onOpenChange={setRuleOpen} rule={ruleEdit?.rule ?? null} onSave={(rule) => { if (ruleEdit) updateRule(ruleEdit.index, rule); else addRule(rule) }} onDelete={ruleEdit ? () => removeRule(ruleEdit.index) : undefined} />
    </>
  )
}
