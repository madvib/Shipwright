interface PreviewSectionProps {
  icon: React.ReactNode
  label: string
  items: string[]
  labels?: string[]
  selected: Set<string>
  onToggle: (key: string) => void
  onToggleAll: (keys: string[]) => void
}

export function PreviewSection({ icon, label, items, labels, selected, onToggle, onToggleAll }: PreviewSectionProps) {
  if (items.length === 0) return null

  const allSelected = items.every((i) => selected.has(i))
  const selectedCount = items.filter((i) => selected.has(i)).length

  return (
    <div className="mb-3 last:mb-0">
      <div className="flex items-center justify-between gap-1.5 mb-1.5">
        <div className="flex items-center gap-1.5">
          <span className="text-muted-foreground">{icon}</span>
          <span className="text-xs font-semibold text-foreground">
            {label}
            <span className="ml-1.5 rounded-full bg-primary/10 px-1.5 py-0.5 text-[10px] font-bold text-primary">
              {selectedCount}/{items.length}
            </span>
          </span>
        </div>
        <button
          onClick={() => onToggleAll(items)}
          className="text-[10px] text-muted-foreground hover:text-foreground transition"
        >
          {allSelected ? 'Deselect all' : 'Select all'}
        </button>
      </div>
      <div className="flex flex-col gap-1 pl-5">
        {items.map((key, i) => {
          const displayName = labels?.[i] ?? key
          return (
            <label
              key={key}
              className="flex items-center gap-2 cursor-pointer group"
            >
              <input
                type="checkbox"
                checked={selected.has(key)}
                onChange={() => onToggle(key)}
                className="size-3 rounded border-border/60 accent-primary cursor-pointer"
              />
              <span className={`text-[11px] font-medium transition ${selected.has(key) ? 'text-foreground' : 'text-muted-foreground line-through'}`}>
                {displayName}
              </span>
            </label>
          )
        })}
      </div>
    </div>
  )
}
