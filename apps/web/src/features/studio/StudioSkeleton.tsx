/**
 * Skeleton loading states for Studio routes.
 * Each skeleton mirrors the layout of its corresponding page
 * so the user sees a stable shimmer while data loads.
 */

function Bone({ className }: { className?: string }) {
  return <div className={`animate-pulse rounded-lg bg-muted ${className ?? ''}`} />
}

/* ── Dashboard ──────────────────────────────────────────────────────── */

export function DashboardSkeleton() {
  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-5xl mx-auto px-6 py-8">
        {/* Header */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <Bone className="h-7 w-28 mb-2" />
            <Bone className="h-4 w-44" />
          </div>
          <Bone className="h-9 w-28 rounded-lg" />
        </div>

        {/* Stat cards */}
        <div className="grid grid-cols-3 gap-3 mb-8">
          {Array.from({ length: 3 }).map((_, i) => (
            <div
              key={i}
              className="rounded-xl border border-border/60 bg-card p-4 flex flex-col items-center justify-center text-center"
            >
              <Bone className="size-8 rounded-lg mb-2" />
              <Bone className="h-7 w-10 mb-1" />
              <Bone className="h-3 w-16" />
            </div>
          ))}
        </div>

        {/* Quick links */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="md:col-span-2 space-y-4">
            {Array.from({ length: 2 }).map((_, i) => (
              <div
                key={i}
                className="flex items-center justify-between rounded-xl border border-border/60 bg-card p-4"
              >
                <div className="flex items-center gap-3">
                  <Bone className="size-9 rounded-lg" />
                  <div>
                    <Bone className="h-4 w-24 mb-1.5" />
                    <Bone className="h-3 w-16" />
                  </div>
                </div>
                <Bone className="size-4 rounded" />
              </div>
            ))}
          </div>

          <div>
            <div className="rounded-xl border border-border/60 bg-card p-4">
              <Bone className="h-3 w-24 mb-3" />
              <div className="space-y-1.5">
                {Array.from({ length: 3 }).map((_, i) => (
                  <Bone key={i} className="h-8 w-full rounded-lg" />
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

/* ── Agent List ─────────────────────────────────────────────────────── */

export function AgentListSkeleton() {
  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-5xl mx-auto px-6 py-8">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div>
            <Bone className="h-7 w-24 mb-2" />
            <Bone className="h-4 w-36" />
          </div>
          <Bone className="h-9 w-28 rounded-lg" />
        </div>

        {/* Agent rows */}
        <div className="space-y-2">
          {Array.from({ length: 4 }).map((_, i) => (
            <div
              key={i}
              className="flex items-center gap-4 rounded-xl border border-border/60 bg-card p-4"
            >
              <Bone className="size-10 shrink-0 rounded-xl" />
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  <Bone className="h-4 w-28" />
                  <Bone className="h-4 w-12 rounded" />
                </div>
                <Bone className="h-3 w-20" />
              </div>
              <Bone className="size-4 rounded" />
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}

/* ── Agent Detail ───────────────────────────────────────────────────── */

export function AgentDetailSkeleton() {
  return (
    <main className="flex-1 overflow-y-auto">
      <div className="mx-auto max-w-[800px]">
        {/* Agent header area */}
        <div className="px-6 py-6 border-b border-border/60">
          <div className="flex items-center gap-4">
            <Bone className="size-12 rounded-xl" />
            <div className="flex-1 min-w-0">
              <Bone className="h-6 w-40 mb-2" />
              <div className="flex items-center gap-2">
                <Bone className="h-4 w-14 rounded" />
                <Bone className="h-4 w-14 rounded" />
              </div>
            </div>
            <Bone className="h-8 w-16 rounded-lg" />
          </div>
        </div>

        {/* Section cards */}
        {Array.from({ length: 4 }).map((_, i) => (
          <div key={i} className="px-6 py-5 border-b border-border/60">
            <div className="flex items-center justify-between mb-4">
              <Bone className="h-5 w-24" />
              <Bone className="h-7 w-16 rounded-lg" />
            </div>
            <div className="space-y-2">
              {Array.from({ length: 2 }).map((_, j) => (
                <Bone key={j} className="h-10 w-full rounded-lg" />
              ))}
            </div>
          </div>
        ))}

        <div className="h-24" />
      </div>
    </main>
  )
}

/* ── Skills IDE ─────────────────────────────────────────────────────── */

export function SkillsIdeSkeleton() {
  return (
    <>
      {/* Mobile placeholder */}
      <div className="flex md:hidden flex-col items-center justify-center gap-4 px-8 py-20 text-center min-h-[60vh]">
        <Bone className="size-12 rounded-xl" />
        <div>
          <Bone className="h-5 w-32 mx-auto mb-2" />
          <Bone className="h-4 w-48 mx-auto" />
        </div>
      </div>

      {/* Desktop 3-panel layout */}
      <div className="hidden md:flex flex-1 h-full min-h-0 overflow-hidden">
        {/* File explorer panel */}
        <div className="w-56 shrink-0 border-r border-border/60 p-3 space-y-2">
          <Bone className="h-8 w-full rounded-md mb-3" />
          {Array.from({ length: 6 }).map((_, i) => (
            <Bone key={i} className="h-6 w-full rounded-md" />
          ))}
        </div>

        {/* Editor panel */}
        <div className="flex-1 flex flex-col min-w-0">
          {/* Tab bar */}
          <div className="flex items-center gap-1 border-b border-border/60 px-2 py-1.5">
            {Array.from({ length: 3 }).map((_, i) => (
              <Bone key={i} className="h-7 w-24 rounded-md" />
            ))}
          </div>
          {/* Editor body */}
          <div className="flex-1 p-4 space-y-2">
            <Bone className="h-4 w-2/3 rounded" />
            <Bone className="h-4 w-5/6 rounded" />
            <Bone className="h-4 w-1/2 rounded" />
            <Bone className="h-4 w-3/4 rounded" />
            <Bone className="h-4 w-2/5 rounded" />
            <Bone className="h-4 w-4/5 rounded" />
            <Bone className="h-4 w-3/5 rounded" />
            <Bone className="h-4 w-full rounded" />
            <Bone className="h-4 w-1/3 rounded" />
            <Bone className="h-4 w-5/6 rounded" />
            <Bone className="h-4 w-2/3 rounded" />
            <Bone className="h-4 w-1/2 rounded" />
          </div>
        </div>

        {/* Preview panel */}
        <div className="w-72 shrink-0 border-l border-border/60 p-4 space-y-3">
          <Bone className="h-5 w-20 mb-2" />
          <Bone className="h-4 w-full" />
          <Bone className="h-4 w-3/4" />
          <Bone className="h-4 w-5/6" />
          <Bone className="h-px w-full bg-border/60 my-3" />
          <Bone className="h-4 w-full" />
          <Bone className="h-4 w-2/3" />
        </div>
      </div>
    </>
  )
}

/* ── Settings ───────────────────────────────────────────────────────── */

export function SettingsSkeleton() {
  return (
    <div className="mx-auto max-w-[680px] px-5 py-6 pb-24">
      {/* Header */}
      <div className="mb-6">
        <Bone className="h-6 w-24 mb-2" />
        <Bone className="h-4 w-60" />
      </div>

      {/* Form sections */}
      {Array.from({ length: 5 }).map((_, i) => (
        <div key={i} className="mb-6 rounded-xl border border-border/60 bg-card p-5">
          <Bone className="h-5 w-32 mb-4" />
          <div className="space-y-3">
            <div>
              <Bone className="h-3 w-20 mb-1.5" />
              <Bone className="h-9 w-full rounded-md" />
            </div>
            <div>
              <Bone className="h-3 w-24 mb-1.5" />
              <Bone className="h-9 w-full rounded-md" />
            </div>
          </div>
        </div>
      ))}
    </div>
  )
}
