import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useState, useCallback, useRef, useEffect, useMemo } from 'react'
import { useAgentStore, makeAgent } from '#/features/agents/useAgentStore'
import { AgentActivityBar, SECTION_DEFS } from '#/features/agents/AgentActivityBar'
import { AgentStickyHeader } from '#/features/agents/AgentStickyHeader'
import { SkillsSection } from '#/features/agents/sections/SkillsSection'
import { McpSection } from '#/features/agents/sections/McpSection'
import { PermissionsSection } from '#/features/agents/sections/PermissionsSection'
import { ProviderSettingsSection } from '#/features/agents/sections/ProviderSettingsSection'
import { RulesSection } from '#/features/agents/sections/RulesSection'
import { AddSkillDialog } from '#/features/agents/dialogs/AddSkillDialog'
import { AddMcpDialog } from '#/features/agents/dialogs/AddMcpDialog'
import { EditAgentDialog } from '#/features/agents/dialogs/EditAgentDialog'
import { PermissionsDialog } from '#/features/agents/dialogs/PermissionsDialog'
import { RuleEditorDialog } from '#/features/agents/dialogs/RuleEditorDialog'
import type { ResolvedAgentProfile, ToolPermission, ToolToggleState } from '#/features/agents/types'
import type { Skill, Rule, ProfilePermissions } from '@ship/ui'

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

  const handleDelete = useCallback(() => {
    if (!confirm(`Delete "${profile.profile.name}"?`)) return
    deleteAgent(id)
    void navigate({ to: '/studio/agents', replace: true })
  }, [id, profile.profile.name, deleteAgent, navigate])

  // ── Convenience mutators (same API as old useAgentDetail) ─────────────

  const update = useCallback(
    (patch: Partial<ResolvedAgentProfile>) => updateAgent(id, patch),
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

  const setPermissionPreset = useCallback(
    (preset: string) => update({ permissions: { ...profile.permissions, preset } }),
    [update, profile.permissions],
  )

  // MCP tool states managed in local component state (not persisted on profile)
  const [mcpToolStates, setMcpToolStates] = useState<Record<string, ToolToggleState>>({})

  const setToolPermission = useCallback(
    (serverName: string, toolName: string, permission: ToolPermission) => {
      setMcpToolStates((prev) => {
        const serverTools = prev[serverName] ?? {}
        return { ...prev, [serverName]: { ...serverTools, [toolName]: permission } }
      })
    },
    [],
  )

  const setGroupPermission = useCallback(
    (serverName: string, toolNames: string[], permission: ToolPermission) => {
      setMcpToolStates((prev) => {
        const serverTools = { ...(prev[serverName] ?? {}) }
        for (const name of toolNames) serverTools[name] = permission
        return { ...prev, [serverName]: serverTools }
      })
    },
    [],
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
    (permissions: ProfilePermissions) => update({ permissions: { ...permissions, preset: 'custom' } }),
    [update],
  )

  // ── Provider settings managed in local state (not on profile) ────────
  const [providerSettings, setProviderSettings] = useState<Record<string, Record<string, unknown>>>({})

  // ── Dialog state ──────────────────────────────────────────────────────

  const [skillOpen, setSkillOpen] = useState(false)
  const [mcpOpen, setMcpOpen] = useState(false)
  const [editOpen, setEditOpen] = useState(false)
  const [permsOpen, setPermsOpen] = useState(false)
  const [ruleOpen, setRuleOpen] = useState(false)
  const [ruleEdit, setRuleEdit] = useState<{ index: number; rule: Rule } | null>(null)

  // ── Scrollspy ─────────────────────────────────────────────────────────

  const scrollRef = useRef<HTMLDivElement>(null)
  const [activeSection, setActiveSection] = useState<string>(SECTION_DEFS[0].id)
  const isScrollingRef = useRef(false)

  useEffect(() => {
    const container = scrollRef.current
    if (!container) return

    const sectionIds = SECTION_DEFS.map((s) => `section-${s.id}`)
    const observer = new IntersectionObserver(
      (entries) => {
        if (isScrollingRef.current) return
        for (const entry of entries) {
          if (entry.isIntersecting) {
            const sectionId = entry.target.id.replace('section-', '')
            setActiveSection(sectionId)
            break
          }
        }
      },
      { root: container, rootMargin: '-10% 0px -80% 0px', threshold: 0 },
    )

    for (const id of sectionIds) {
      const el = container.querySelector(`#${id}`)
      if (el) observer.observe(el)
    }

    return () => observer.disconnect()
  }, [])

  const handleSectionClick = useCallback((sectionId: string) => {
    const container = scrollRef.current
    if (!container) return
    const el = container.querySelector(`#section-${sectionId}`)
    if (!el) return

    isScrollingRef.current = true
    setActiveSection(sectionId)
    el.scrollIntoView({ behavior: 'smooth', block: 'start' })

    // Re-enable scrollspy after animation completes
    setTimeout(() => {
      isScrollingRef.current = false
    }, 600)
  }, [])

  // ── Section counts for activity bar badges ────────────────────────────

  const counts = useMemo(() => ({
    skills: profile.skills.length,
    mcp: profile.mcpServers.length,
    rules: profile.rules.length,
  }), [
    profile.skills.length,
    profile.mcpServers.length,
    profile.rules.length,
  ])

  return (
    <>
      <div className="flex h-full min-h-0 overflow-hidden">
        {/* Activity bar — hidden on mobile */}
        <AgentActivityBar
          activeSection={activeSection}
          onSectionClick={handleSectionClick}
          counts={counts}
        />

        {/* Content area */}
        <div className="flex-1 min-w-0 flex flex-col overflow-hidden">
          <AgentStickyHeader
            profile={profile}
            onEdit={() => setEditOpen(true)}
            onDelete={handleDelete}
          />

          <div ref={scrollRef} className="flex-1 overflow-y-auto">
            <div id="section-skills">
              <SkillsSection
                skills={profile.skills}
                onRemove={removeSkill}
                onAdd={() => setSkillOpen(true)}
              />
            </div>

            <div id="section-mcp">
              <McpSection
                servers={profile.mcpServers}
                toolStates={mcpToolStates}
                onRemove={removeServer}
                onSetToolPermission={setToolPermission}
                onSetGroupPermission={setGroupPermission}
                onAdd={() => setMcpOpen(true)}
              />
            </div>

            <div id="section-permissions">
              <PermissionsSection
                permissions={profile.permissions ?? {}}
                activePreset={profile.permissions?.preset ?? 'ship-standard'}
                onPresetChange={setPermissionPreset}
                onEdit={() => setPermsOpen(true)}
              />
            </div>

            <div id="section-providers">
              <ProviderSettingsSection
                providers={['claude', 'gemini', 'codex', 'cursor', 'opencode']}
                providerSettings={providerSettings}
                onChange={setProviderSettings}
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

            <div className="h-24" />
          </div>
        </div>
      </div>

      {/* Dialogs — outside layout flow */}
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

      <EditAgentDialog
        open={editOpen}
        onOpenChange={setEditOpen}
        profile={profile}
        onSave={(patch) => update(patch)}
      />

      <PermissionsDialog
        open={permsOpen}
        onOpenChange={setPermsOpen}
        permissions={profile.permissions ?? {}}
        onSave={updatePermissions}
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
    </>
  )
}
