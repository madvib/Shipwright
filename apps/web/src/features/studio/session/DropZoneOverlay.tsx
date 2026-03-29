// Full-screen overlay shown when dragging files over the session page.
// Visual cue: dashed border + centered label.

import { Upload } from 'lucide-react'

export function DropZoneOverlay() {
  return (
    <div className="absolute inset-0 z-30 flex items-center justify-center bg-background/60 backdrop-blur-sm pointer-events-none">
      <div className="flex flex-col items-center gap-3 rounded-xl border-2 border-dashed border-primary/50 bg-card/80 px-12 py-10">
        <Upload className="size-8 text-primary/70" />
        <p className="text-sm font-medium text-foreground">Drop to add to session</p>
        <p className="text-xs text-muted-foreground">Images, HTML, Markdown, or any file</p>
      </div>
    </div>
  )
}
