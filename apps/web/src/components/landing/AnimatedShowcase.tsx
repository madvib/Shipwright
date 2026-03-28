import { useState, useEffect } from 'react'
import { ProviderLogo } from '#/features/compiler/ProviderLogo'
import { CodeOutput } from './animated-showcase-helpers'

// ── Agent data ───────────────────────────────────────────────────────────────

const AGENTS = [
  {
    name: 'web-lane',
    preset: 'autonomous',
    skills: ['task-coordination', 'code-review', 'frontend-design'],
    mcp: [
      { name: 'project', tools: 'all' },
      { name: 'github', tools: '8/18' },
      { name: 'filesystem', tools: '4/8' },
    ],
    permissions: {
      allow: ['Read', 'Grep', 'Glob', 'Bash(git *)'],
      deny: ['Bash(rm -rf *)', 'Bash(git push --force*)'],
    },
    rules: ['no-compat.md', 'test-policy.md'],
  },
  {
    name: 'reviewer',
    preset: 'readonly',
    skills: ['code-review'],
    mcp: [{ name: 'project', tools: 'all' }],
    permissions: {
      allow: ['Read', 'Grep', 'Glob'],
      deny: ['Write(*)', 'Edit(*)', 'Bash(rm*)'],
    },
    rules: ['review-checklist.md'],
  },
  {
    name: 'ops',
    preset: 'elevated',
    skills: ['task-coordination', 'deploy-tools', 'monitor'],
    mcp: [
      { name: 'project', tools: 'all' },
      { name: 'github', tools: '14/18' },
    ],
    permissions: {
      allow: ['Read', 'Grep', 'Bash(git push*)', 'Bash(deploy*)'],
      deny: ['Bash(rm -rf /)'],
    },
    rules: ['deploy-safety.md', 'rollback-protocol.md'],
  },
]

const PROVIDERS = ['claude', 'cursor', 'gemini', 'codex'] as const
const PROVIDER_FILES: Record<string, string> = {
  claude: 'CLAUDE.md',
  cursor: '.cursor/rules/',
  gemini: 'GEMINI.md',
  codex: 'AGENTS.md',
}

// ── Component ────────────────────────────────────────────────────────────────

export function AnimatedShowcase() {
  const [agentIdx, setAgentIdx] = useState(0)
  const [providerIdx, setProviderIdx] = useState(0)
  const [phase, setPhase] = useState<'idle' | 'compiling' | 'done'>('done')

  // Cycle agents
  useEffect(() => {
    const timer = setInterval(() => {
      setPhase('compiling')
      setTimeout(() => {
        setAgentIdx((p) => (p + 1) % AGENTS.length)
        setPhase('done')
      }, 600)
    }, 5000)
    return () => clearInterval(timer)
  }, [])

  // Cycle provider tabs
  useEffect(() => {
    const timer = setInterval(() => {
      setProviderIdx((p) => (p + 1) % PROVIDERS.length)
    }, 2200)
    return () => clearInterval(timer)
  }, [])

  const agent = AGENTS[agentIdx]
  const provider = PROVIDERS[providerIdx]

  return (
    <section className="mx-auto mb-20 max-w-[62rem] px-6 sm:px-10">
      <div className="overflow-hidden rounded-2xl border border-border/60 shadow-2xl shadow-black/20 bg-card/20">
        {/* Browser bar */}
        <div className="flex items-center gap-1.5 border-b border-border/60 bg-muted/40 px-4 py-2.5">
          <span className="size-2.5 rounded-full bg-red-500/50" />
          <span className="size-2.5 rounded-full bg-amber-500/50" />
          <span className="size-2.5 rounded-full bg-emerald-500/50" />
          <span className="flex-1 text-center font-mono text-[11px] text-muted-foreground/50">
            $ agent use{' '}
            <span className={`transition-opacity duration-300 ${phase === 'compiling' ? 'opacity-40' : ''}`}>
              {agent.name}
            </span>
          </span>
        </div>

        <div className="flex flex-col sm:flex-row">
          {/* ── Left: Agent config ──────────────────────────────────────── */}
          <div className="flex-1 min-w-0 p-5 sm:p-6 border-b sm:border-b-0 sm:border-r border-border/40 sm:h-[340px] overflow-hidden">
            {/* Agent selector tabs */}
            <div className="flex gap-1 mb-4">
              {AGENTS.map((a, i) => (
                <button
                  key={a.name}
                  onClick={() => { setAgentIdx(i); setPhase('compiling'); setTimeout(() => setPhase('done'), 600) }}
                  className={`rounded-md px-2.5 py-1 text-[10px] font-semibold transition-all duration-300 ${
                    i === agentIdx
                      ? 'bg-primary/15 text-primary'
                      : 'text-muted-foreground/40 hover:text-muted-foreground'
                  }`}
                >
                  {a.name}
                </button>
              ))}
            </div>

            {/* Config sections */}
            <div className={`space-y-3.5 transition-opacity duration-300 min-h-[220px] ${phase === 'compiling' ? 'opacity-30' : ''}`}>
              {/* Skills */}
              <div>
                <div className="mb-1 text-[8px] font-semibold uppercase tracking-wider text-muted-foreground/40">Skills</div>
                <div className="flex flex-wrap gap-1">
                  {agent.skills.map((s) => (
                    <span key={s} className="rounded-md border border-border/50 bg-muted/20 px-2 py-0.5 text-[10px] text-muted-foreground">
                      {s}
                    </span>
                  ))}
                </div>
              </div>

              {/* MCP */}
              <div>
                <div className="mb-1 text-[8px] font-semibold uppercase tracking-wider text-muted-foreground/40">MCP Servers</div>
                <div className="flex flex-wrap gap-1">
                  {agent.mcp.map((m) => (
                    <span key={m.name} className="flex items-center gap-1.5 rounded-md border border-border/50 bg-muted/20 px-2 py-0.5 text-[10px] text-muted-foreground">
                      {m.name}
                      <span className="text-[8px] text-emerald-400">{m.tools}</span>
                    </span>
                  ))}
                </div>
              </div>

              {/* Permissions */}
              <div>
                <div className="mb-1 text-[8px] font-semibold uppercase tracking-wider text-muted-foreground/40">
                  Permissions
                  <span className="ml-1.5 text-[7px] font-normal normal-case text-primary/60">{agent.preset}</span>
                </div>
                <div className="space-y-0.5">
                  {agent.permissions.allow.map((p) => (
                    <div key={p} className="flex items-center gap-1.5 text-[10px]">
                      <span className="size-1.5 rounded-full bg-emerald-400" />
                      <span className="font-mono text-muted-foreground">{p}</span>
                    </div>
                  ))}
                  {agent.permissions.deny.map((p) => (
                    <div key={p} className="flex items-center gap-1.5 text-[10px]">
                      <span className="size-1.5 rounded-full bg-red-400" />
                      <span className="font-mono text-muted-foreground/40 line-through">{p}</span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Rules */}
              <div>
                <div className="mb-1 text-[8px] font-semibold uppercase tracking-wider text-muted-foreground/40">Rules</div>
                <div className="flex flex-wrap gap-1">
                  {agent.rules.map((r) => (
                    <span key={r} className="rounded border border-border/40 bg-muted/10 px-1.5 py-0.5 text-[9px] font-mono text-muted-foreground/60">
                      {r}
                    </span>
                  ))}
                </div>
              </div>
            </div>
          </div>

          {/* ── Right: Compiled output ──────────────────────────────────── */}
          <div className="w-full sm:w-[280px] shrink-0 p-5 sm:p-6 flex flex-col sm:h-[340px]">
            {/* Provider tabs */}
            <div className="flex gap-1.5 mb-3">
              {PROVIDERS.map((p, i) => (
                <button
                  key={p}
                  onClick={() => setProviderIdx(i)}
                  className={`flex items-center gap-1 rounded-md px-2 py-1 text-[10px] font-medium transition-all duration-300 ${
                    i === providerIdx
                      ? 'bg-muted/60 text-foreground'
                      : 'text-muted-foreground/30 hover:text-muted-foreground/60'
                  }`}
                >
                  <ProviderLogo provider={p} />
                </button>
              ))}
            </div>

            {/* File name */}
            <div className="text-[9px] font-mono text-muted-foreground/40 mb-2">
              {PROVIDER_FILES[provider]}
            </div>

            {/* Code output */}
            <div className={`flex-1 rounded-lg border border-border/40 bg-background/60 p-3 font-mono text-[9px] leading-relaxed min-h-[180px] transition-opacity duration-300 ${
              phase === 'compiling' ? 'opacity-20' : ''
            }`}>
              <CodeOutput agent={agent} provider={provider} />
            </div>

            {/* Provider output badges */}
            <div className="mt-3 flex flex-wrap gap-1">
              {PROVIDERS.map((p, i) => (
                <span
                  key={p}
                  className={`flex items-center gap-1 rounded border border-border/40 px-1.5 py-0.5 text-[8px] transition-all duration-500 ${
                    phase === 'done' ? 'opacity-100' : 'opacity-20'
                  }`}
                  style={{ transitionDelay: phase === 'done' ? `${i * 80}ms` : '0ms' }}
                >
                  <span className="size-1 rounded-full bg-emerald-400" />
                  <span className="text-muted-foreground/60">{PROVIDER_FILES[p]}</span>
                </span>
              ))}
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}
