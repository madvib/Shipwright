import { Link } from '@tanstack/react-router'
import { Zap, Terminal, Package } from 'lucide-react'

const PROVIDERS = [
  { abbr: 'C', name: 'Claude Code', bg: 'bg-primary/10', text: 'text-primary' },
  { abbr: 'G', name: 'Gemini CLI', bg: 'bg-blue-500/10', text: 'text-blue-400' },
  { abbr: 'O', name: 'Codex CLI', bg: 'bg-emerald-500/10', text: 'text-emerald-400' },
  { abbr: 'Cu', name: 'Cursor', bg: 'bg-violet-500/10', text: 'text-violet-400' },
]

export function LandingHero() {
  return (
    <section className="px-6 pb-10 pt-36 text-center sm:px-10 sm:pt-40">
      <div className="mx-auto max-w-[56rem]">
        {/* Badge */}
        <div className="mb-6 inline-flex items-center gap-1.5 rounded-full border border-primary/20 bg-primary/8 px-3.5 py-1.5 text-[11px] font-medium text-primary">
          <Package className="size-3" />
          v0.1.0 -- now in public beta
        </div>

        {/* Headline */}
        <h1 className="mb-4 font-display text-5xl font-extrabold leading-[1.08] tracking-[-0.03em] sm:text-7xl">
          One config.
          <br />
          <span className="text-primary">Every agent.</span>
        </h1>

        {/* Subhead */}
        <p className="mx-auto mb-8 max-w-lg text-lg leading-relaxed text-muted-foreground">
          The package manager for AI coding agents. Define skills, permissions,
          and MCP servers once -- compile to Claude, Gemini, Codex, and Cursor.
        </p>

        {/* CTAs */}
        <div className="mb-4 flex flex-wrap items-center justify-center gap-3">
          <Link
            to="/studio"
            className="inline-flex items-center gap-2 rounded-xl bg-primary px-7 py-3 text-sm font-semibold text-primary-foreground no-underline transition hover:-translate-y-0.5 hover:bg-primary/90"
          >
            <Zap className="size-4" />
            Open Studio
          </Link>
          <a
            href="https://github.com/madvib/Ship"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-2 rounded-xl border border-border bg-transparent px-7 py-3 text-sm font-semibold text-muted-foreground no-underline transition hover:-translate-y-0.5 hover:border-border/80 hover:text-foreground"
          >
            <Terminal className="size-4" />
            View on GitHub
          </a>
        </div>
        <p className="text-[11px] text-muted-foreground/50">
          Free and open source. No account required.
        </p>
      </div>

      {/* Provider strip */}
      <div className="mt-10 flex flex-wrap items-center justify-center gap-8 sm:gap-10">
        {PROVIDERS.map((p) => (
          <div
            key={p.name}
            className="flex items-center gap-2 text-[13px] font-medium text-muted-foreground transition hover:text-foreground/70"
          >
            <span
              className={`flex size-7 items-center justify-center rounded-lg text-xs font-bold ${p.bg} ${p.text}`}
            >
              {p.abbr}
            </span>
            {p.name}
          </div>
        ))}
      </div>
    </section>
  )
}
