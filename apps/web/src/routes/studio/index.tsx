import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useEffect } from 'react'
import { DashboardSkeleton } from '#/features/studio/StudioSkeleton'

export const Route = createFileRoute('/studio/')({
  component: StudioRedirect,
  pendingComponent: DashboardSkeleton,
})

function StudioRedirect() {
  const navigate = useNavigate()

  useEffect(() => {
    void navigate({ to: '/studio/agents', replace: true })
  }, [])

  return <DashboardSkeleton />
}
