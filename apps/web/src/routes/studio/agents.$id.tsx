import { createFileRoute } from '@tanstack/react-router'
import { useState } from 'react'
import { useAgentDetail } from '../../features/agents/useAgentDetail'
import { AgentHeader } from '../../features/agents/sections/AgentHeader'
import { SkillsSection } from '../../features/agents/sections/SkillsSection'
import { McpSection } from '../../features/agents/sections/McpSection'
import { SubagentsSection } from '../../features/agents/sections/SubagentsSection'
import { PermissionsSection } from '../../features/agents/sections/PermissionsSection'
import { SettingsSection } from '../../features/agents/sections/SettingsSection'
import { HooksSection } from '../../features/agents/sections/HooksSection'
import { RulesSection } from '../../features/agents/sections/RulesSection'
import { AddSkillDialog } from '../../features/agents/dialogs/AddSkillDialog'
import { AddMcpDialog } from '../../features/agents/dialogs/AddMcpDialog'
import { AddSubagentDialog } from '../../features/agents/dialogs/AddSubagentDialog'

export const Route = createFileRoute('/studio/agents/$id')({
  component: AgentDetailPage,
  ssr: false,
})

function AgentDetailPage() {
  const { id } = Route.useParams()
  const agent = useAgentDetail(id)
  const [skillOpen, setSkillOpen] = useState(false)
  const [mcpOpen, setMcpOpen] = useState(false)
  const [subagentOpen, setSubagentOpen] = useState(false)

  return (
    <main className="flex-1 overflow-y-auto">
      <div className="mx-auto max-w-[800px]">
        <AgentHeader profile={agent.profile} />

        <SkillsSection
          skills={agent.profile.skills}
          onRemove={agent.removeSkill}
          onAdd={() => setSkillOpen(true)}
        />

        <McpSection
          servers={agent.profile.mcpServers}
          toolStates={agent.profile.mcpToolStates}
          onRemove={agent.removeServer}
          onSetToolPermission={agent.setToolPermission}
          onSetGroupPermission={agent.setGroupPermission}
          onAdd={() => setMcpOpen(true)}
        />

        <SubagentsSection
          subagents={agent.profile.subagents}
          onRemove={agent.removeSubagent}
          onAdd={() => setSubagentOpen(true)}
        />

        <PermissionsSection
          permissions={agent.profile.permissions}
          activePreset={agent.profile.permissionPreset}
          onPresetChange={agent.setPermissionPreset}
        />

        <SettingsSection
          settings={agent.profile.settings}
          onUpdate={agent.updateSettings}
        />

        <HooksSection hooks={agent.profile.hooks} />
        <RulesSection rules={agent.profile.rules} />

        <div className="h-24" />
      </div>

      <AddSkillDialog
        open={skillOpen}
        onOpenChange={setSkillOpen}
        existingIds={agent.profile.skills.map((s) => s.id)}
        onAdd={agent.addSkill}
      />

      <AddMcpDialog
        open={mcpOpen}
        onOpenChange={setMcpOpen}
        existingNames={agent.profile.mcpServers.map((s) => s.name)}
        onAdd={(server) => agent.updateProfile({ mcpServers: [...agent.profile.mcpServers, server] })}
      />

      <AddSubagentDialog
        open={subagentOpen}
        onOpenChange={setSubagentOpen}
        currentAgentId={id}
        existingIds={agent.profile.subagents.map((s) => s.id)}
        onAdd={(ref) => agent.updateProfile({ subagents: [...agent.profile.subagents, ref] })}
      />
    </main>
  )
}
