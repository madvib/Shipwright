import { useState, useEffect } from 'react'
import { ProviderLogo } from '#/features/compiler/ProviderLogo'

const PROVIDERS = [
  { id: 'claude', file: 'CLAUDE.md', color: '#d4a274' },
  { id: 'cursor', file: '.cursor/rules/', color: '#fff' },
  { id: 'gemini', file: 'GEMINI.md', color: '#4285f4' },
  { id: 'codex', file: 'AGENTS.md', color: '#10a37f' },
]

const AGENT_CONFIGS = [
  { name: 'web-lane', skills: 5, mcp: 3, preset: 'autonomous', color: '#f59e0b' },
  { name: 'reviewer', skills: 1, mcp: 1, preset: 'readonly', color: '#3b82f6' },
  { name: 'ops', skills: 3, mcp: 2, preset: 'elevated', color: '#ef4444' },
]

export function AnimatedShowcase() {
  const [activeAgent, setActiveAgent] = useState(0)
  const [compiling, setCompiling] = useState(false)
  const [compiled, setCompiled] = useState(false)

  // Cycle through agents
  useEffect(() => {
    const interval = setInterval(() => {
      setCompiling(true)
      setCompiled(false)
      setTimeout(() => {
        setCompiling(false)
        setCompiled(true)
        setTimeout(() => {
          setActiveAgent((p) => (p + 1) % AGENT_CONFIGS.length)
          setCompiled(false)
        }, 2000)
      }, 800)
    }, 4000)
    // Trigger first compile immediately
    setCompiling(true)
    setTimeout(() => { setCompiling(false); setCompiled(true) }, 800)
    return () => clearInterval(interval)
  }, [])

  const agent = AGENT_CONFIGS[activeAgent]

  return (
    <section className="mx-auto mb-20 max-w-[56rem] px-6 sm:px-10">
      <div className="overflow-hidden rounded-2xl border border-border/60 shadow-2xl shadow-black/20 bg-card/30">
        {/* Top bar */}
        <div className="flex items-center gap-1.5 border-b border-border/60 bg-muted/40 px-4 py-2.5">
          <span className="size-2.5 rounded-full bg-red-500/50" />
          <span className="size-2.5 rounded-full bg-amber-500/50" />
          <span className="size-2.5 rounded-full bg-emerald-500/50" />
          <span className="flex-1 text-center text-[11px] text-muted-foreground/50">
            ship use {agent.name}
          </span>
        </div>

        <div className="p-6 sm:p-8">
          {/* Flow: Agent selector → compile → provider outputs */}
          <div className="flex flex-col sm:flex-row items-center gap-6 sm:gap-8">

            {/* Left: Agent input */}
            <div className="w-full sm:w-48 shrink-0">
              <div className="text-[9px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-2">
                .ship/agents/
              </div>
              <div className="space-y-1.5">
                {AGENT_CONFIGS.map((a, i) => (
                  <div
                    key={a.name}
                    className={`flex items-center gap-2.5 rounded-lg border px-3 py-2 transition-all duration-500 ${
                      i === activeAgent
                        ? 'border-primary/40 bg-primary/5 scale-[1.02]'
                        : 'border-border/40 bg-transparent opacity-40'
                    }`}
                  >
                    <div
                      className="size-6 rounded-md flex items-center justify-center text-[9px] font-bold text-white"
                      style={{ background: a.color }}
                    >
                      {a.name.charAt(0).toUpperCase()}
                    </div>
                    <div className="min-w-0">
                      <div className="text-[11px] font-semibold text-foreground truncate">{a.name}</div>
                      <div className="text-[9px] text-muted-foreground/50">
                        {a.skills} skills · {a.mcp} MCP · {a.preset}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Center: Compile indicator */}
            <div className="flex flex-col items-center gap-2 shrink-0">
              <div
                className={`size-10 rounded-xl flex items-center justify-center transition-all duration-300 ${
                  compiling
                    ? 'bg-primary/20 scale-110'
                    : compiled
                      ? 'bg-emerald-500/20'
                      : 'bg-muted/40'
                }`}
              >
                {compiling ? (
                  <div className="size-4 border-2 border-primary/60 border-t-primary rounded-full animate-spin" />
                ) : compiled ? (
                  <svg className="size-5 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                  </svg>
                ) : (
                  <svg className="size-5 text-muted-foreground/30" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M13 7l5 5m0 0l-5 5m5-5H6" />
                  </svg>
                )}
              </div>
              <span className={`text-[9px] font-semibold transition-colors duration-300 ${
                compiling ? 'text-primary' : compiled ? 'text-emerald-400' : 'text-muted-foreground/30'
              }`}>
                {compiling ? 'compiling...' : compiled ? 'done' : 'compile'}
              </span>
            </div>

            {/* Right: Provider outputs */}
            <div className="flex-1 w-full min-w-0">
              <div className="text-[9px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-2">
                output
              </div>
              <div className="grid grid-cols-2 gap-2">
                {PROVIDERS.map((p, i) => (
                  <div
                    key={p.id}
                    className={`flex items-center gap-2 rounded-lg border border-border/40 bg-background/40 px-3 py-2 transition-all duration-500 ${
                      compiled ? 'opacity-100 translate-y-0' : 'opacity-20 translate-y-1'
                    }`}
                    style={{ transitionDelay: compiled ? `${i * 100}ms` : '0ms' }}
                  >
                    <ProviderLogo provider={p.id} size="sm" />
                    <div className="min-w-0">
                      <div className="text-[10px] font-medium text-foreground truncate">{p.file}</div>
                      <div className="text-[8px] text-muted-foreground/40">.mcp.json + rules</div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}
