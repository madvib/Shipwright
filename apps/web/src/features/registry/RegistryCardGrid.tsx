import { useState } from 'react'
import { Link } from '@tanstack/react-router'
import { Download, Star, Zap } from 'lucide-react'
import type { RegistryCard } from './registry-cards'
import { formatInstalls, TYPE_ICON_STYLES, FEATURED_COLLECTION } from './registry-cards'

// ── Featured banner ──────────────────────────────────────────────────────────

export function FeaturedBanner() {
  const [installing, setInstalling] = useState(false)

  const handleInstallPack = () => {
    setInstalling(true)
    console.log('[Registry] Install pack requested: superpowers-skill-pack')
    setTimeout(() => setInstalling(false), 1200)
  }

  return (
    <div className="flex items-center gap-4 rounded-xl border border-primary/15 bg-gradient-to-r from-primary/5 to-transparent p-5 mb-7">
      <div className="flex size-12 shrink-0 items-center justify-center rounded-xl bg-gradient-to-br from-primary to-primary/70">
        <Zap className="size-6 text-primary-foreground" />
      </div>
      <div className="flex-1 min-w-0">
        <div className="text-[9px] font-semibold text-primary uppercase tracking-wider mb-0.5">
          {FEATURED_COLLECTION.badge}
        </div>
        <div className="text-base font-bold text-foreground mb-0.5">
          {FEATURED_COLLECTION.title}
        </div>
        <div className="text-xs text-muted-foreground">
          {FEATURED_COLLECTION.description}
        </div>
      </div>
      <div className="shrink-0 relative">
        <button
          onClick={handleInstallPack}
          disabled={installing}
          className="inline-flex items-center gap-1.5 rounded-lg bg-primary px-4 py-2 text-xs font-semibold text-primary-foreground transition hover:opacity-90 disabled:opacity-60"
        >
          {installing ? 'Installing...' : 'Install pack'}
        </button>
        {/* Orange dot: install flow not wired */}
        <span
          className="absolute -top-1 -right-1 size-2 rounded-full bg-status-orange"
          title="Full install flow not yet wired"
        />
      </div>
    </div>
  )
}

// ── Card section ─────────────────────────────────────────────────────────────

export function CardSection({
  title,
  cards,
  installedIds,
  onInstall,
}: {
  title: string
  cards: RegistryCard[]
  installedIds: Set<string>
  onInstall: (id: string) => void
}) {
  return (
    <div className="mb-7">
      <div className="text-[11px] font-semibold text-muted-foreground/50 uppercase tracking-wider mb-3 pl-1">
        {title}
      </div>
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2.5">
        {cards.map((card) => (
          <RegistryPackageCard
            key={card.id}
            card={card}
            isInstalled={installedIds.has(card.id)}
            onInstall={() => onInstall(card.id)}
          />
        ))}
      </div>
    </div>
  )
}

// ── Package card ─────────────────────────────────────────────────────────────

function RegistryPackageCard({
  card,
  isInstalled,
  onInstall,
}: {
  card: RegistryCard
  isInstalled: boolean
  onInstall: () => void
}) {
  const iconStyle = TYPE_ICON_STYLES[card.type]

  return (
    <Link
      to={`/studio/registry/${card.id}` as '/'}
      className="group flex flex-col rounded-[10px] border border-border/60 bg-card p-4 transition-all duration-150 hover:border-border hover:shadow-md hover:shadow-foreground/[0.03] hover:-translate-y-px no-underline"
    >
      {/* Top: icon + info */}
      <div className="flex items-start gap-2.5 mb-2.5">
        <div
          className={`flex size-9 shrink-0 items-center justify-center rounded-lg text-sm font-bold ${iconStyle}`}
        >
          {card.icon}
        </div>
        <div className="flex-1 min-w-0">
          <div className="text-[13px] font-semibold text-foreground leading-tight group-hover:text-primary transition-colors">
            {card.name}
          </div>
          <div className="text-[10px] text-muted-foreground/50">{card.author}</div>
        </div>
      </div>

      {/* Description */}
      <p className="text-[11px] text-muted-foreground leading-relaxed mb-2.5 flex-1 line-clamp-2">
        {card.description}
      </p>

      {/* Footer: stats + install */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2.5 text-[10px] text-muted-foreground/50">
          <span className="flex items-center gap-1">
            <Download className="size-[11px]" />
            {formatInstalls(card.installs)}
          </span>
          {card.type === 'skill' && (
            <span className="flex items-center gap-1">
              <Star className="size-[11px]" />
              {card.rating}
            </span>
          )}
          {card.type === 'agent' && (
            <>
              {card.skillCount != null && <span>{card.skillCount} skills</span>}
              {card.mcpCount != null && <span>{card.mcpCount} MCP</span>}
            </>
          )}
          {card.type === 'mcp' && card.toolCount != null && (
            <span>{card.toolCount} tools</span>
          )}
        </div>
        <div className="relative">
          <button
            onClick={(e) => {
              e.preventDefault()
              e.stopPropagation()
              onInstall()
            }}
            className={`rounded-md border px-2.5 py-1 text-[10px] font-medium transition-colors ${
              isInstalled
                ? 'text-status-green border-status-green/20'
                : 'text-muted-foreground border-border/60 hover:border-primary hover:text-primary'
            }`}
          >
            {isInstalled ? 'Installed' : 'Install'}
          </button>
          {/* Orange dot: install flow not fully wired */}
          {!isInstalled && (
            <span
              className="absolute -top-0.5 -right-0.5 size-1.5 rounded-full bg-status-orange"
              title="Full install flow not yet wired"
            />
          )}
        </div>
      </div>
    </Link>
  )
}
