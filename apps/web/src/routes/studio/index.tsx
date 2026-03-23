import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useEffect } from 'react'
import { useAgentStore } from '#/features/agents/useAgentStore'
import { AGENT_TEMPLATES, templateToAgent } from '#/features/agents/agent-templates'
import { DashboardSkeleton } from '#/features/studio/StudioSkeleton'

export const Route = createFileRoute('/studio/')({
  component: StudioRedirect,
  pendingComponent: DashboardSkeleton,
})

function StudioRedirect() {
  const { agents, createAgent } = useAgentStore()
  const navigate = useNavigate()

  useEffect(() => {
    if (agents.length > 0) {
      // Returning user → agents list
      void navigate({ to: '/studio/agents', replace: true })
    } else {
      // First visit → create agent from default template and open it
      const template = AGENT_TEMPLATES[0]
      const id = createAgent(templateToAgent(template, template.name))
      void navigate({ to: '/studio/agents/$id', params: { id }, replace: true })
    }
  }, [])

  return <DashboardSkeleton />
}
