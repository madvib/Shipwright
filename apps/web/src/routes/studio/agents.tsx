import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/studio/agents')({
  component: AgentsLayout,
  ssr: false,
})

function AgentsLayout() {
  return <Outlet />
}
