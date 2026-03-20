import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/studio/workflow')({ component: WorkflowSketch })

function WorkflowSketch() {
  if (!import.meta.env.DEV) return null
  return (
    <div className="flex-1 overflow-auto p-8">
      <div className="max-w-3xl mx-auto">
        <span className="text-xs font-mono text-muted-foreground/40 border border-dashed border-muted-foreground/20 rounded px-2 py-0.5">dev only</span>
        <h1 className="text-xl font-bold mt-4 mb-2">Workflow (v2 sketch)</h1>
        <p className="text-sm text-muted-foreground">Multi-agent orchestration canvas. Not shipped in v0.1.0.</p>
      </div>
    </div>
  )
}
