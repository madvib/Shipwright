export function EditorSkeleton() {
  return (
    <div className="flex flex-1 flex-col min-w-0">
      <div className="flex items-center gap-1 border-b border-border px-2 py-1.5 shrink-0">
        {Array.from({ length: 2 }).map((_, i) => (
          <div key={i} className="h-6 w-20 animate-pulse rounded bg-muted" />
        ))}
      </div>
      <div className="flex-1 p-4 space-y-2">
        {Array.from({ length: 10 }).map((_, i) => (
          <div key={i} className="h-4 animate-pulse rounded bg-muted" style={{ width: `${40 + (i * 7) % 50}%` }} />
        ))}
      </div>
    </div>
  )
}

export function EmptyState({ icon, title, subtitle, children }: {
  icon: React.ReactNode; title: string; subtitle: string; children?: React.ReactNode
}) {
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-3 text-center text-muted-foreground min-w-0 px-6">
      {icon}
      <div>
        <p className="text-sm font-medium text-foreground">{title}</p>
        <p className="mt-1 text-xs text-muted-foreground">{subtitle}</p>
        {children}
      </div>
    </div>
  )
}
