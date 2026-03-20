import { createFileRoute } from '@tanstack/react-router'
import { useAgentDetail } from '../../features/agents/useAgentDetail'
import { AgentHeader } from '../../features/agents/sections/AgentHeader'
import { SkillsSection } from '../../features/agents/sections/SkillsSection'
import { McpSection } from '../../features/agents/sections/McpSection'
import { SubagentsSection } from '../../features/agents/sections/SubagentsSection'
import { PermissionsSection } from '../../features/agents/sections/PermissionsSection'
import { SettingsSection } from '../../features/agents/sections/SettingsSection'
import { HooksSection } from '../../features/agents/sections/HooksSection'
import { RulesSection } from '../../features/agents/sections/RulesSection'

export const Route = createFileRoute('/studio/agents/$id')({
  component: AgentDetailPage,
})

function AgentDetailPage() {
  const { id } = Route.useParams()
  const {
    profile,
    removeSkill,
    removeServer,
    removeSubagent,
    setPermissionPreset,
    updateSettings,
    setToolPermission,
    setGroupPermission,
  } = useAgentDetail(id)

  return (
    <main className="flex-1 overflow-y-auto">
      <div className="mx-auto max-w-[800px]">
        <AgentHeader profile={profile} />

        <SkillsSection
          skills={profile.skills}
          onRemove={removeSkill}
        />

        <McpSection
          servers={profile.mcpServers}
          toolStates={profile.mcpToolStates}
          onRemove={removeServer}
          onSetToolPermission={setToolPermission}
          onSetGroupPermission={setGroupPermission}
        />

        <SubagentsSection
          subagents={profile.subagents}
          onRemove={removeSubagent}
        />

        <PermissionsSection
          permissions={profile.permissions}
          activePreset={profile.permissionPreset}
          onPresetChange={setPermissionPreset}
        />

        <SettingsSection
          settings={profile.settings}
          onUpdate={updateSettings}
        />

        <HooksSection hooks={profile.hooks} />

        <RulesSection rules={profile.rules} />

        {/* Bottom spacer for scroll clearance */}
        <div className="h-24" />
      </div>
    </main>
  )
}
