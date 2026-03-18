import { useState } from 'react'
import { createFileRoute } from '@tanstack/react-router'
import { WorkflowGallery } from '#/components/canvas/WorkflowGallery'
import { WorkflowCanvas } from '#/components/canvas/WorkflowCanvas'
import { PRESETS } from '#/components/canvas/presets'
import type { WorkflowPreset } from '#/components/canvas/types'

export const Route = createFileRoute('/studio/workflow')({
  component: WorkflowPage,
})

function WorkflowPage() {
  const [selected, setSelected] = useState<{ preset: WorkflowPreset; name: string } | null>(null)

  const handleSelect = (presetId: string) => {
    const info = PRESETS.find((p) => p.id === presetId)
    if (!info) return
    setSelected({ preset: info.load(), name: info.name })
  }

  // Editing mode — overlay fullscreen canvas
  if (selected) {
    return (
      <div className="fixed inset-0 z-40 flex flex-col bg-background">
        <WorkflowCanvas
          preset={selected.preset}
          presetName={selected.name}
          onBack={() => setSelected(null)}
        />
      </div>
    )
  }

  // Gallery mode — renders inside normal studio layout with header + dock
  return (
    <div className="h-full overflow-auto">
      <WorkflowGallery onSelect={handleSelect} />
    </div>
  )
}
