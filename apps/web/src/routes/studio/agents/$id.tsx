import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useState, useCallback, useMemo, useEffect } from 'react'
import { useAgentStore } from '#/features/agents/useAgentStore'
import { useAgentEditor } from '#/features/agents/useAgentEditor'
import { AgentActivityBar } from '#/features/agents/AgentActivityBar'
import { AgentStickyHeader } from '#/features/agents/AgentStickyHeader'
import { useScrollspy } from '#/features/agents/useScrollspy'
import { SkillsSection } from '#/features/agents/sections/SkillsSection'
import { McpSection } from '#/features/agents/sections/McpSection'
import { PermissionsSection } from '#/features/agents/sections/PermissionsSection'
import { RulesSection } from '#/features/agents/sections/RulesSection'
import { ModelSection } from '#/features/agents/sections/ModelSection'
import { AddSkillDialog } from '#/features/agents/dialogs/AddSkillDialog'
import { AddMcpDialog } from '#/features/agents/dialogs/AddMcpDialog'
import { EditAgentDialog } from '#/features/agents/dialogs/EditAgentDialog'
import { PermissionsDialog } from '#/features/agents/dialogs/PermissionsDialog'
import { RuleEditorDialog } from '#/features/agents/dialogs/RuleEditorDialog'
import { ProviderModal } from '#/features/agents/dialogs/ProviderModal'
import type { ToolPermission } from '#/features/agents/types'
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
  const store = useAgentStore()
  const editor = useAgentEditor(id)
  const { scrollRef, activeSection, handleSectionClick } = useScrollspy()

  // ── beforeunload — panic save dirty state ────────────────────────────
  useEffect(() => {
    if (!editor.meta.isDirty) return
    const handler = (e: BeforeUnloadEvent) => {
      editor.panicSave()
      e.preventDefault()
    }
    window.addEventListener('beforeunload', handler)
    return () => window.removeEventListener('beforeunload', handler)
  }, [editor.meta.isDirty, editor.panicSave])

  // ── Delete ───────────────────────────────────────────────────────────
  const handleDelete = useCallback(() => {
    if (!confirm(`Delete "${editor.agent.profile.name}"?`)) return
    store.deleteAgent(id)
    void navigate({ to: '/studio/agents', replace: true })
  }, [id, editor.agent.profile.name, store, navigate])

  // ── Dialog state ─────────────────────────────────────────────────────
  const [skillOpen, setSkillOpen] = useState(false)
  const [mcpOpen, setMcpOpen] = useState(false)
  const [editOpen, setEditOpen] = useState(false)
  const [permsOpen, setPermsOpen] = useState(false)
  const [ruleOpen, setRuleOpen] = useState(false)
  const [ruleEdit, setRuleEdit] = useState<{ index: number; rule: Rule } | null>(null)
  const [providerModalOpen, setProviderModalOpen] = useState<string | null>(null)

  // ── Navigation guard dialog ──────────────────────────────────────────
  const [navGuardOpen, setNavGuardOpen] = useState(false)
  const [pendingNav, setPendingNav] = useState<(() => void) | null>(null)

  const guardedNavigate = useCallback(
    (navigateFn: () => void) => {
      if (editor.meta.isDirty) {
        setPendingNav(() => navigateFn)
        setNavGuardOpen(true)
      } else {
        navigateFn()
      }
    },
    [editor.meta.isDirty],
  )

  // ── Counts for activity bar ──────────────────────────────────────────
  const counts = useMemo(() => ({
    skills: editor.agent.skills.length,
    mcp: editor.agent.mcpServers.length,
    rules: editor.agent.rules.length,
  }), [editor.agent.skills.length, editor.agent.mcpServers.length, editor.agent.rules.length])

  // ── Provider hooks filtering ─────────────────────────────────────────
  const getProviderHooks = useCallback(
    (provider: string): HookConfig[] =>
      editor.agent.hooks.filter((h) => (h as HookConfig & { provider?: string }).provider === provider || (!(h as HookConfig & { provider?: string }).provider && provider === 'claude')),
    [editor.agent.hooks],
  )

  const setProviderHooks = useCallback(
    (provider: string, newHooks: HookConfig[]) => {
      // Replace hooks for this provider, keep others
      const tagged = newHooks.map((h) => ({ ...h, provider } as HookConfig & { provider: string }))
      const others = editor.agent.hooks.filter((h) => {
        const p = (h as HookConfig & { provider?: string }).provider
        return p ? p !== provider : provider !== 'claude'
      })
      editor.setHooks([...others, ...tagged] as HookConfig[])
    },
    [editor],
  )

  return (
    <>
      <div className="flex h-full min-h-0 overflow-hidden">
        <AgentActivityBar activeSection={activeSection} onSectionClick={handleSectionClick} counts={counts} />
        <div className="flex-1 min-w-0 flex flex-col overflow-hidden">
          <AgentStickyHeader
            agent={editor.agent}
            meta={editor.meta}
            onSave={editor.save}
            onNameChange={(name) => editor.updateProfile({ name })}
            onEdit={() => setEditOpen(true)}
            onDelete={handleDelete}
            onOpenProviderModal={setProviderModalOpen}
          />
          <div ref={scrollRef} className="flex-1 overflow-y-auto">
            <div id="section-skills">
              <SkillsSection skills={editor.agent.skills} onRemove={editor.removeSkill} onAdd={() => setSkillOpen(true)} />
            </div>
            <div id="section-mcp">
              <McpSection servers={editor.agent.mcpServers} toolStates={editor.agent.toolPermissions ?? {}} onRemove={editor.removeMcpServer} onSetToolPermission={editor.setToolPermission} onSetGroupPermission={editor.setGroupPermission} onAdd={() => setMcpOpen(true)} />
            </div>
            <div id="section-permissions">
              <PermissionsSection
                permissions={editor.agent.permissions ?? {}}
                activePreset={editor.agent.permissions?.preset ?? 'ship-standard'}
                onPresetChange={(preset) => editor.setPermissions({ ...editor.agent.permissions, preset })}
                onEdit={() => setPermsOpen(true)}
              />
            </div>
            <div id="section-rules">
              <RulesSection
                rules={editor.agent.rules}
                onAdd={() => { setRuleEdit(null); setRuleOpen(true) }}
                onEdit={(i) => { setRuleEdit({ index: i, rule: editor.agent.rules[i] }); setRuleOpen(true) }}
                onRemove={editor.removeRule}
              />
            </div>
            <div id="section-model">
              <ModelSection
                model={(editor.agent as ResolvedAgentProfileWithModel)._model ?? ''}
                onChange={editor.setModel}
              />
            </div>
            <div className="h-24" />
          </div>
        </div>
      </div>

      <AddSkillDialog open={skillOpen} onOpenChange={setSkillOpen} existingIds={editor.agent.skills.map((s) => s.id)} onAdd={editor.addSkill} />
      <AddMcpDialog open={mcpOpen} onOpenChange={setMcpOpen} existingNames={editor.agent.mcpServers.map((s) => s.name)} onAdd={editor.addMcpServer} />
      <EditAgentDialog open={editOpen} onOpenChange={setEditOpen} profile={editor.agent} onSave={(patch) => {
        if (patch.profile) editor.updateProfile(patch.profile)
      }} />
      <PermissionsDialog open={permsOpen} onOpenChange={setPermsOpen} permissions={editor.agent.permissions ?? {}} onSave={(perms) => editor.setPermissions({ ...perms, preset: 'custom' })} />
      <RuleEditorDialog
        open={ruleOpen}
        onOpenChange={setRuleOpen}
        rule={ruleEdit?.rule ?? null}
        onSave={(rule) => { if (ruleEdit) editor.updateRule(ruleEdit.index, rule as Rule); else editor.addRule(rule as Rule) }}
        onDelete={ruleEdit ? () => editor.removeRule(ruleEdit.index) : undefined}
      />

      {/* Provider modal */}
      <ProviderModal
        open={providerModalOpen !== null}
        onOpenChange={(open) => { if (!open) setProviderModalOpen(null) }}
        provider={providerModalOpen ?? 'claude'}
        settings={editor.agent.providerSettings?.[providerModalOpen ?? 'claude'] ?? {}}
        hooks={getProviderHooks(providerModalOpen ?? 'claude')}
        onSettingsChange={(s) => editor.setProviderSettings(providerModalOpen ?? 'claude', s)}
        onHooksChange={(h) => setProviderHooks(providerModalOpen ?? 'claude', h)}
      />

      {/* Navigation guard dialog */}
      {navGuardOpen && (
        <>
          <div className="fixed inset-0 z-[60] bg-black/50 backdrop-blur-sm" />
          <div className="fixed inset-0 z-[60] flex items-center justify-center p-4">
            <div role="dialog" aria-modal="true" className="w-full max-w-xs rounded-xl border border-border/60 bg-card shadow-2xl p-5 space-y-4">
              <h3 className="font-display text-sm font-semibold">Unsaved changes</h3>
              <p className="text-xs text-muted-foreground">You have unsaved changes. What would you like to do?</p>
              <div className="flex gap-2">
                <button
                  onClick={() => { setNavGuardOpen(false); pendingNav?.() }}
                  className="flex-1 rounded-lg border border-border/60 px-3 py-2 text-xs font-medium text-muted-foreground hover:text-foreground transition"
                >
                  Discard
                </button>
                <button
                  onClick={() => { editor.save(); setNavGuardOpen(false); pendingNav?.() }}
                  className="flex-1 rounded-lg bg-primary px-3 py-2 text-xs font-medium text-primary-foreground transition hover:opacity-90"
                >
                  Save & Leave
                </button>
              </div>
              <button
                onClick={() => { setNavGuardOpen(false); setPendingNav(null) }}
                className="w-full text-center text-[11px] text-muted-foreground/50 hover:text-muted-foreground transition"
              >
                Stay
              </button>
            </div>
          </div>
        </>
      )}
    </>
  )
}

// Type hack for the _model virtual field added by the editor
type ResolvedAgentProfileWithModel = import('#/features/agents/types').ResolvedAgentProfile & { _model?: string }
