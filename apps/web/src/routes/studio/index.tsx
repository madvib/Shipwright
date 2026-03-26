import { createFileRoute, Navigate } from '@tanstack/react-router'
import { useEffect, useState } from 'react'
import { useAgentStore } from '#/features/agents/useAgentStore'
import { AGENT_TEMPLATES, templateToAgent } from '#/features/agents/agent-templates'
import { DashboardSkeleton } from '#/features/studio/StudioSkeleton'

export const Route = createFileRoute('/studio/')({
  component: StudioRedirect,
  pendingComponent: DashboardSkeleton,
})

function StudioRedirect() {
  const { agents, createAgent } = useAgentStore()
  const [createdId, setCreatedId] = useState<string | null>(null)

  // First visit: create a default agent (side effect in useEffect, not render)
  useEffect(() => {
    if (agents.length === 0 && !createdId) {
      const template = AGENT_TEMPLATES[0]
      const id = createAgent(templateToAgent(template, template.name))
      setCreatedId(id)
    }
  }, [agents.length, createdId, createAgent])

  // Returning user → agents list
  if (agents.length > 0 && !createdId) {
    return <Navigate to="/studio/agents" replace />
  }

  // First visit → open the just-created agent
  if (createdId) {
    return <Navigate to="/studio/agents/$id" params={{ id: createdId }} replace />
  }

  return <DashboardSkeleton />
}
