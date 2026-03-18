import { createFileRoute, Link } from '@tanstack/react-router'
import { useProfiles } from '#/features/studio/useProfiles'
import { useLibrary } from '#/features/compiler/useLibrary'
import { TechIcon } from '#/features/studio/TechIcon'
import { PROVIDERS } from '#/features/compiler/types'

export const Route = createFileRoute('/studio/')({ component: StudioDashboard })

function StudioDashboard() {
  const { profiles } = useProfiles()
  const { library, selectedProviders } = useLibrary()

  const hasProfiles = profiles.length > 0
  const skillCount = library.skills?.length ?? 0
  const mcpCount = library.mcp_servers?.length ?? 0

  return (
    <div className="h-full flex flex-col">
      <div className="flex-1 overflow-auto p-5 pb-20">

        {/* Import banner — shown when no profiles yet */}
        {!hasProfiles && (
          <div className="mb-5 flex items-center justify-between gap-6 rounded-xl border border-violet-500/20 bg-violet-500/5 px-5 py-4">
            <div>
              <p className="text-sm font-semibold text-foreground mb-0.5">Import an existing project</p>
              <p className="text-[11px] text-muted-foreground">
                Ship reads your repo — CLAUDE.md, .mcp.json, GEMINI.md, AGENTS.md, .cursor/, .codex/ — and consolidates everything into <code className="rounded bg-muted/50 px-1 font-mono text-[10px]">.ship/</code>
              </p>
            </div>
            <div className="flex gap-2 shrink-0">
              <Link
                to="/studio/profiles"
                className="h-8 px-4 rounded-lg bg-violet-600 hover:bg-violet-500 transition-colors text-xs font-medium text-white flex items-center no-underline"
              >
                Import
              </Link>
              <Link
                to="/studio/profiles"
                className="h-8 px-4 rounded-lg border border-border/60 bg-card hover:border-border transition-colors text-xs text-muted-foreground flex items-center no-underline"
              >
                Start blank
              </Link>
            </div>
          </div>
        )}

        {/* Dashboard grid — fills viewport */}
        <div className="grid grid-cols-3 grid-rows-[minmax(0,1fr)_minmax(0,1fr)] gap-3 min-h-[calc(100vh-180px)]">

          {/* Profiles — large, spans 2 cols */}
          <Link
            to="/studio/profiles"
            className="group col-span-2 rounded-xl border border-border/60 bg-card p-5 hover:border-violet-500/40 transition-colors no-underline flex flex-col"
          >
            <div className="flex items-center justify-between mb-4">
              <span className="text-[10px] font-semibold uppercase tracking-widest text-violet-400">Profiles</span>
              <span className="text-[10px] text-muted-foreground/40 group-hover:text-muted-foreground transition-colors">&#8250;</span>
            </div>
            {hasProfiles ? (
              <div className="flex-1 flex flex-col">
                <div className="grid grid-cols-2 xl:grid-cols-3 gap-3 flex-1">
                  {profiles.slice(0, 6).map((p) => (
                    <div
                      key={p.id}
                      className="rounded-lg border bg-background/50 p-3 flex items-start gap-3"
                      style={{ borderColor: p.accentColor + '30' }}
                    >
                      <TechIcon stack={p.icon} size={28} style={{ borderRadius: 6 }} />
                      <div className="min-w-0 flex-1">
                        <p className="text-xs font-semibold text-foreground truncate">{p.name}</p>
                        <p className="text-[10px] text-muted-foreground mt-0.5">
                          {p.skills.length} skill{p.skills.length !== 1 ? 's' : ''} · {p.mcpServers.length} MCP
                        </p>
                        <div className="flex gap-1 mt-1.5 flex-wrap">
                          {p.selectedProviders.slice(0, 3).map((pid) => (
                            <span key={pid} className="text-[8px] bg-muted rounded px-1 py-0.5 text-muted-foreground/60">{pid}</span>
                          ))}
                        </div>
                      </div>
                    </div>
                  ))}
                  {profiles.length > 6 && (
                    <div className="rounded-lg border border-dashed border-border/40 flex items-center justify-center text-[11px] text-muted-foreground/40">
                      + {profiles.length - 6} more
                    </div>
                  )}
                </div>
                <p className="text-[11px] text-violet-400 mt-3">Manage profiles &#8250;</p>
              </div>
            ) : (
              <div className="flex-1 flex flex-col justify-center items-center text-center">
                <p className="text-sm text-muted-foreground mb-2">No profiles yet</p>
                <p className="text-[11px] text-violet-400">Create your first profile &#8250;</p>
              </div>
            )}
          </Link>

          {/* Providers */}
          <Link
            to="/studio/profiles"
            className="group rounded-xl border border-border/60 bg-card p-5 hover:border-primary/30 transition-colors no-underline flex flex-col"
          >
            <div className="flex items-center justify-between mb-4">
              <span className="text-[10px] font-semibold uppercase tracking-widest text-primary">Providers</span>
              <span className="text-[10px] text-muted-foreground/40 group-hover:text-muted-foreground transition-colors">&#8250;</span>
            </div>
            <div className="flex-1 flex flex-col gap-2">
              {PROVIDERS.map((p) => {
                const active = selectedProviders.includes(p.id)
                return (
                  <div
                    key={p.id}
                    className={`rounded-lg border px-3 py-2.5 transition-colors ${
                      active
                        ? 'border-primary/20 bg-primary/5'
                        : 'border-border/40 bg-muted/20 opacity-50'
                    }`}
                  >
                    <div className="flex items-center gap-2">
                      <span className={`size-1.5 rounded-full ${active ? 'bg-emerald-500' : 'bg-muted-foreground/30'}`} />
                      <span className="text-xs font-semibold text-foreground">{p.name}</span>
                    </div>
                    <p className="text-[10px] text-muted-foreground mt-1 pl-3.5">{p.description}</p>
                    {active && (
                      <div className="flex gap-1 mt-1.5 pl-3.5 flex-wrap">
                        {p.files.slice(0, 2).map((f) => (
                          <span key={f} className="font-mono text-[8px] bg-muted rounded px-1 py-0.5 text-muted-foreground/60">{f}</span>
                        ))}
                      </div>
                    )}
                  </div>
                )
              })}
            </div>
          </Link>

          {/* Skills */}
          <Link
            to="/studio/skills"
            className="group rounded-xl border border-border/60 bg-card p-5 hover:border-emerald-500/30 transition-colors no-underline flex flex-col"
          >
            <div className="flex items-center justify-between mb-4">
              <span className="text-[10px] font-semibold uppercase tracking-widest text-emerald-400">Skills</span>
              <span className="text-[10px] text-muted-foreground/40 group-hover:text-muted-foreground transition-colors">&#8250;</span>
            </div>
            <p className="text-2xl font-bold text-foreground mb-1">{skillCount}</p>
            <p className="text-[11px] text-muted-foreground mb-3">in collection</p>
            {skillCount > 0 ? (
              <div className="flex-1 flex flex-col gap-1">
                {library.skills.slice(0, 5).map((s) => (
                  <div key={s.id} className="flex items-center gap-2 rounded px-2 py-1 bg-muted/30">
                    <span className="size-1 rounded-full bg-emerald-500/60" />
                    <span className="text-[10px] text-foreground/80 truncate">{s.name}</span>
                  </div>
                ))}
                {skillCount > 5 && (
                  <p className="text-[10px] text-muted-foreground/50 pl-2 mt-1">+ {skillCount - 5} more</p>
                )}
              </div>
            ) : (
              <div className="flex-1 flex items-end">
                <p className="text-[11px] text-emerald-400">Browse library &#8250;</p>
              </div>
            )}
          </Link>

          {/* MCP Servers */}
          <Link
            to="/studio/mcp"
            className="group rounded-xl border border-border/60 bg-card p-5 hover:border-sky-500/30 transition-colors no-underline flex flex-col"
          >
            <div className="flex items-center justify-between mb-4">
              <span className="text-[10px] font-semibold uppercase tracking-widest text-sky-400">MCP Servers</span>
              <span className="text-[10px] text-muted-foreground/40 group-hover:text-muted-foreground transition-colors">&#8250;</span>
            </div>
            <p className="text-2xl font-bold text-foreground mb-1">{mcpCount}</p>
            <p className="text-[11px] text-muted-foreground mb-3">configured</p>
            {mcpCount > 0 ? (
              <div className="flex-1 flex flex-col gap-1">
                {library.mcp_servers.slice(0, 5).map((s) => (
                  <div key={s.name} className="flex items-center gap-2 rounded px-2 py-1 bg-muted/30">
                    <span className="size-1 rounded-full bg-sky-500/60" />
                    <span className="font-mono text-[10px] text-foreground/80 truncate">{s.name}</span>
                  </div>
                ))}
                {mcpCount > 5 && (
                  <p className="text-[10px] text-muted-foreground/50 pl-2 mt-1">+ {mcpCount - 5} more</p>
                )}
              </div>
            ) : (
              <div className="flex-1 flex items-end">
                <p className="text-[11px] text-sky-400">Explore registry &#8250;</p>
              </div>
            )}
          </Link>

          {/* Export */}
          {hasProfiles ? (
            <Link
              to="/studio/export"
              className="group rounded-xl border border-border/60 bg-card p-5 hover:border-border transition-colors no-underline flex flex-col"
            >
              <div className="flex items-center justify-between mb-4">
                <span className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">Export</span>
                <span className="text-[10px] text-muted-foreground/40 group-hover:text-muted-foreground transition-colors">&#8250;</span>
              </div>
              <p className="text-[11px] text-muted-foreground mb-3">Generate provider config files</p>
              <div className="flex-1 flex flex-col justify-end">
                <div className="flex flex-wrap gap-1.5">
                  {PROVIDERS.map((p) => {
                    const active = selectedProviders.includes(p.id)
                    return (
                      <span
                        key={p.id}
                        className={`rounded px-2 py-1 text-[10px] font-medium ${
                          active
                            ? 'bg-primary/10 text-primary'
                            : 'bg-muted text-muted-foreground/40'
                        }`}
                      >
                        {p.name.split(' ')[0]}
                      </span>
                    )
                  })}
                </div>
              </div>
            </Link>
          ) : (
            <div className="rounded-xl border border-border/40 bg-card/50 p-5 opacity-40 flex flex-col">
              <div className="mb-4">
                <span className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">Export</span>
              </div>
              <p className="text-[11px] text-muted-foreground/60">All provider files</p>
              <p className="text-[11px] text-muted-foreground/40 mt-1">Nothing to export</p>
            </div>
          )}

        </div>
      </div>
    </div>
  )
}
