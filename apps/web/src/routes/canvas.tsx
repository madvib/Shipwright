import { createFileRoute } from '@tanstack/react-router'
import { WorkflowCanvas } from '../components/canvas/WorkflowCanvas'

export const Route = createFileRoute('/canvas')({
  component: CanvasPage,
})

function CanvasPage() {
  return (
    <div className="flex flex-col" style={{ height: 'calc(100vh - 53px)' }}>
      <WorkflowCanvas />
    </div>
  )
}
