// Loading skeleton for the Session page, matching its two-panel layout.

function Bone({ className }: { className?: string }) {
  return <div className={`animate-pulse rounded-lg bg-muted ${className ?? ''}`} />
}

export function SessionSkeleton() {
  return (
    <>
      {/* Mobile placeholder */}
      <div className="flex md:hidden flex-col items-center justify-center gap-4 px-8 py-20 text-center min-h-[60vh]">
        <Bone className="size-12 rounded-xl" />
        <div>
          <Bone className="h-5 w-36 mx-auto mb-2" />
          <Bone className="h-4 w-48 mx-auto" />
        </div>
      </div>

      {/* Desktop layout */}
      <div className="hidden md:flex flex-1 h-full min-h-0 overflow-hidden">
        {/* Canvas area */}
        <div className="flex-1 flex flex-col min-w-0">
          {/* Toolbar */}
          <div className="flex items-center gap-2 border-b border-border/60 px-3 py-1.5">
            <Bone className="h-6 w-20 rounded-md" />
            <Bone className="h-6 w-16 rounded-md" />
          </div>
          {/* Canvas body */}
          <div className="flex-1 p-8 flex items-center justify-center">
            <Bone className="w-2/3 h-2/3 rounded-xl" />
          </div>
        </div>

        {/* Timeline sidebar */}
        <div className="w-64 shrink-0 border-l border-border/60 p-3 space-y-3">
          <Bone className="h-4 w-16 mb-2" />
          {Array.from({ length: 5 }).map((_, i) => (
            <Bone key={i} className="h-10 w-full rounded-md" />
          ))}
        </div>
      </div>
    </>
  )
}
