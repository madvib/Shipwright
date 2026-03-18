import { createFileRoute, Link } from '@tanstack/react-router'
import { useProfiles } from '#/features/studio/useProfiles'
import { useLibrary } from '#/features/compiler/useLibrary'
import { TechIcon } from '#/features/studio/TechIcon'

export const Route = createFileRoute('/studio/')({ component: StudioDashboard })

function StudioDashboard() {
  const { profiles } = useProfiles()
  const { library } = useLibrary()

  const hasProfiles = profiles.length > 0
  const skillCount = library.skills?.length ?? 0
  const mcpCount = library.mcp_servers?.length ?? 0

  return (
    <div className="h-full flex flex-col">
      {/* View header */}
      <div className="flex items-center px-4 h-11 border-b border-border/60 bg-card/30 shrink-0">
        <span className="text-sm font-semibold text-foreground">Studio</span>
      </div>

      <div className="flex-1 overflow-auto px-6 py-6">
        <div className="mx-auto max-w-3xl">

          {/* Import banner — shown when no profiles yet */}
          {!hasProfiles && (
            <div className="mb-6 flex items-center justify-between gap-6 rounded-xl border border-violet-500/20 bg-violet-500/5 px-5 py-4">
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

          {/* Card grid */}
          <div className="grid grid-cols-3 gap-3">

            {/* Profiles */}
            <Link
              to="/studio/profiles"
              className="group rounded-xl border border-border/60 bg-card p-4 hover:border-violet-500/40 transition-colors no-underline block"
            >
              <div className="flex items-center justify-between mb-3">
                <span className="text-[10px] font-semibold uppercase tracking-widest text-violet-400">Profiles</span>
                <span className="text-[10px] text-muted-foreground/40 group-hover:text-muted-foreground transition-colors">&#8250;</span>
              </div>
              {hasProfiles ? (
                <>
                  <p className="text-2xl font-bold text-foreground mb-1">{profiles.length}</p>
                  <p className="text-[11px] text-muted-foreground mb-3">agent presets</p>
                  <div className="flex flex-col gap-1.5">
                    {profiles.slice(0, 2).map((p) => (
                      <div key={p.id} className="flex items-center gap-2">
                        <TechIcon stack={p.icon} size={16} style={{ borderRadius: 4 }} />
                        <span className="text-[11px] text-foreground/80 truncate">{p.name}</span>
                      </div>
                    ))}
                    {profiles.length > 2 && (
                      <p className="text-[11px] text-muted-foreground/50 pl-5">+ {profiles.length - 2} more</p>
                    )}
                  </div>
                </>
              ) : (
                <>
                  <p className="text-[11px] text-muted-foreground mb-4">No profiles yet</p>
                  <p className="text-[11px] text-violet-400">Create first &#8250;</p>
                </>
              )}
            </Link>

            {/* Workflow — locked until profiles exist */}
            {hasProfiles ? (
              <Link
                to="/canvas"
                className="group rounded-xl border border-amber-500/20 bg-card p-4 hover:border-amber-500/40 transition-colors no-underline block"
              >
                <div className="flex items-center justify-between mb-3">
                  <span className="text-[10px] font-semibold uppercase tracking-widest text-amber-400">Workflow</span>
                </div>
                <p className="text-[11px] text-muted-foreground mb-4">Visual orchestration</p>
                <p className="text-[11px] text-amber-400">Open canvas &#8250;</p>
              </Link>
            ) : (
              <div className="rounded-xl border border-border/40 bg-card/50 p-4 opacity-40">
                <div className="mb-3">
                  <span className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">Workflow</span>
                </div>
                <p className="text-[11px] text-muted-foreground/60 mb-1">Visual orchestration</p>
                <p className="text-[11px] text-muted-foreground/40">Add profiles first</p>
              </div>
            )}

            {/* Skills */}
            <Link
              to="/studio/skills"
              className="group rounded-xl border border-border/60 bg-card p-4 hover:border-emerald-500/30 transition-colors no-underline block"
            >
              <div className="flex items-center justify-between mb-3">
                <span className="text-[10px] font-semibold uppercase tracking-widest text-emerald-400">Skills</span>
                <span className="text-[10px] text-muted-foreground/40 group-hover:text-muted-foreground transition-colors">&#8250;</span>
              </div>
              <p className="text-2xl font-bold text-foreground mb-1">{skillCount}</p>
              <p className="text-[11px] text-muted-foreground mb-3">in collection</p>
              <p className="text-[11px] text-emerald-400">Browse library &#8250;</p>
            </Link>

            {/* MCP Servers */}
            <Link
              to="/studio/mcp"
              className="group rounded-xl border border-border/60 bg-card p-4 hover:border-sky-500/30 transition-colors no-underline block"
            >
              <div className="flex items-center justify-between mb-3">
                <span className="text-[10px] font-semibold uppercase tracking-widest text-sky-400">MCP Servers</span>
                <span className="text-[10px] text-muted-foreground/40 group-hover:text-muted-foreground transition-colors">&#8250;</span>
              </div>
              <p className="text-2xl font-bold text-foreground mb-1">{mcpCount}</p>
              <p className="text-[11px] text-muted-foreground mb-3">configured</p>
              <p className="text-[11px] text-sky-400">Explore registry &#8250;</p>
            </Link>

            {/* Export */}
            {hasProfiles ? (
              <Link
                to="/studio/export"
                className="group rounded-xl border border-border/60 bg-card p-4 hover:border-border transition-colors no-underline block"
              >
                <div className="flex items-center justify-between mb-3">
                  <span className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">Export</span>
                  <span className="text-[10px] text-muted-foreground/40 group-hover:text-muted-foreground transition-colors">&#8250;</span>
                </div>
                <p className="text-[11px] text-muted-foreground mb-3">All provider files</p>
                <div className="flex flex-wrap gap-1">
                  {['Claude', 'Gemini', 'Codex', 'Cursor'].map((p) => (
                    <span key={p} className="rounded px-1.5 py-0.5 text-[9px] bg-muted text-muted-foreground">{p}</span>
                  ))}
                </div>
              </Link>
            ) : (
              <div className="rounded-xl border border-border/40 bg-card/50 p-4 opacity-40">
                <div className="mb-3">
                  <span className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">Export</span>
                </div>
                <p className="text-[11px] text-muted-foreground/60">All provider files</p>
                <p className="text-[11px] text-muted-foreground/40 mt-1">Nothing to export</p>
              </div>
            )}

            {/* Presets */}
            <Link
              to="/studio/presets"
              className="group rounded-xl border border-border/60 bg-card p-4 hover:border-border transition-colors no-underline block"
            >
              <div className="flex items-center justify-between mb-3">
                <span className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">Presets</span>
                <span className="text-[10px] text-muted-foreground/40 group-hover:text-muted-foreground transition-colors">&#8250;</span>
              </div>
              <p className="text-[11px] text-muted-foreground mb-3">Community configs</p>
              <p className="text-[11px] text-muted-foreground">Browse &#8250;</p>
            </Link>

          </div>
        </div>
      </div>
    </div>
  )
}
