import { useState } from 'react'
import { createFileRoute } from '@tanstack/react-router'
import { WorkflowGallery } from '../components/canvas/WorkflowGallery'
import { WorkflowCanvas } from '../components/canvas/WorkflowCanvas'
import { PRESETS } from '../components/canvas/presets'
import type { WorkflowPreset } from '../components/canvas/types'

export const Route = createFileRoute('/canvas')({
  component: CanvasPage,
})

function CanvasPage() {
  const [selected, setSelected] = useState<{ preset: WorkflowPreset; name: string } | null>(null)

  const handleSelect = (presetId: string) => {
    const info = PRESETS.find((p) => p.id === presetId)
    if (!info) return
    setSelected({ preset: info.load(), name: info.name })
  }

  if (selected) {
    return (
      <div className="flex flex-col h-screen">
        <WorkflowCanvas
          preset={selected.preset}
          presetName={selected.name}
          onBack={() => setSelected(null)}
        />
      </div>
    )
  }

  return (
    <div className="flex flex-col h-screen">
      <WorkflowGallery onSelect={handleSelect} />
    </div>
  )
}
