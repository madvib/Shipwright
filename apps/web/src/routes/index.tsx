import { createFileRoute } from '@tanstack/react-router'
import { useState, useEffect, useRef } from 'react'
import { Layers, Zap, Bot, ArrowRight, ChevronDown, Sparkles, Check } from 'lucide-react'
import { ProviderLogo } from '../features/compiler/ProviderLogo'

export const Route = createFileRoute('/')({ component: HomePage })

// ── Compiler animation data ────────────────────────────────────────────────

const INPUT_LINES = [
  { text: 'mcp_servers:',           indent: 0, color: 'text-violet-400' },
  { text: '  - github',             indent: 0, color: 'text-foreground/70' },
  { text: '  - linear',             indent: 0, color: 'text-foreground/70' },
  { text: '  - memory',             indent: 0, color: 'text-foreground/70' },
  { text: 'skills:',                indent: 0, color: 'text-violet-400' },
  { text: '  - commit-conventions', indent: 0, color: 'text-foreground/70' },
  { text: '  - code-review',        indent: 0, color: 'text-foreground/70' },
  { text: 'rules:',                 indent: 0, color: 'text-violet-400' },
  { text: '  - code-style.md',      indent: 0, color: 'text-foreground/70' },
  { text: 'providers:',             indent: 0, color: 'text-violet-400' },
  { text: '  - claude',             indent: 0, color: 'text-amber-400/80' },
  { text: '  - gemini',             indent: 0, color: 'text-blue-400/80' },
  { text: '  - codex',              indent: 0, color: 'text-foreground/70' },
  { text: '  - cursor',             indent: 0, color: 'text-emerald-400/80' },
]

const OUTPUT_FILES = [
  { provider: 'claude',  file: 'CLAUDE.md',          ms:  4 },
  { provider: 'claude',  file: '.mcp.json',           ms:  5 },
  { provider: 'gemini',  file: 'GEMINI.md',           ms:  6 },
  { provider: 'gemini',  file: '.gemini/settings.json', ms:  7 },
  { provider: 'codex',   file: 'AGENTS.md',           ms:  8 },
  { provider: 'cursor',  file: '.cursor/mcp.json',    ms:  9 },
  { provider: 'cursor',  file: '.cursor/rules/',      ms: 11 },
]

type Phase = 'typing' | 'compiling' | 'done' | 'pause'

function CompilerAnimation() {
  const [phase, setPhase] = useState<Phase>('typing')
  const [inputCount, setInputCount] = useState(0)
  const [outputCount, setOutputCount] = useState(0)
  const [opacity, setOpacity] = useState(1)
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const schedule = (fn: () => void, ms: number) => {
    timeoutRef.current = setTimeout(fn, ms)
  }

  useEffect(() => {
    return () => { if (timeoutRef.current) clearTimeout(timeoutRef.current) }
  }, [])

  // Typing phase — reveal input lines
  useEffect(() => {
    if (phase !== 'typing') return
    if (inputCount < INPUT_LINES.length) {
      schedule(() => setInputCount(n => n + 1), 80)
    } else {
      schedule(() => setPhase('compiling'), 500)
    }
  }, [phase, inputCount])

  // Compiling phase — hold for engine animation
  useEffect(() => {
    if (phase !== 'compiling') return
    schedule(() => setPhase('done'), 1100)
  }, [phase])

  // Done phase — reveal output files
  useEffect(() => {
    if (phase !== 'done') return
    if (outputCount < OUTPUT_FILES.length) {
      schedule(() => setOutputCount(n => n + 1), 120)
    } else {
      schedule(() => setPhase('pause'), 300)
    }
  }, [phase, outputCount])

  // Pause then restart
  useEffect(() => {
    if (phase !== 'pause') return
    schedule(() => {
      setOpacity(0)
      schedule(() => {
        setInputCount(0)
        setOutputCount(0)
        setPhase('typing')
        setOpacity(1)
      }, 500)
    }, 3500)
  }, [phase])

  const isCompiling = phase === 'compiling'
  const isDone = phase === 'done' || phase === 'pause'

  return (
    <div
      className="mx-auto mt-14 max-w-4xl overflow-hidden rounded-2xl border border-border/60 shadow-2xl shadow-black/20 transition-opacity duration-500"
      style={{ opacity }}
    >
      {/* Title bar */}
      <div className="flex items-center gap-2 border-b border-border/60 bg-muted/40 px-4 py-2.5 backdrop-blur-sm">
        <div className="flex gap-1.5">
          <span className="size-3 rounded-full bg-red-500/40" />
          <span className="size-3 rounded-full bg-amber-500/40" />
          <span className="size-3 rounded-full bg-emerald-500/40" />
        </div>
        <span className="ml-2 font-mono text-[11px] text-muted-foreground">ship-studio · compiler</span>
        {isDone && (
          <span className="ml-auto flex items-center gap-1 text-[10px] text-emerald-500 animate-in fade-in duration-300">
            <Check className="size-3" />
            7 files · 11ms
          </span>
        )}
      </div>

      {/* Body: input | engine | output */}
      <div className="grid min-h-[260px] grid-cols-[1fr_72px_1fr]">
        {/* Input */}
        <div className="bg-[oklch(0.13_0.01_270)] p-5">
          <p className="mb-3 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60">
            library.yml
          </p>
          <div className="space-y-0.5 font-mono text-[12px] leading-5">
            {INPUT_LINES.slice(0, inputCount).map((line, i) => (
              <div
                key={i}
                className={`${line.color} animate-in fade-in slide-in-from-left-1 duration-150`}
              >
                {line.text}
              </div>
            ))}
            {phase === 'typing' && inputCount < INPUT_LINES.length && (
              <span className="inline-block h-3.5 w-1.5 animate-pulse bg-primary/60" />
            )}
          </div>
        </div>

        {/* Engine column */}
        <div className="flex flex-col items-center justify-center gap-3 border-x border-border/60 bg-muted/10">
          <div
            className={`flex size-10 items-center justify-center rounded-xl border transition-all duration-500 ${
              isCompiling
                ? 'border-primary/60 bg-primary/15 shadow-[0_0_20px_oklch(0.7_0.2_58_/_35%)]'
                : isDone
                ? 'border-emerald-500/40 bg-emerald-500/10'
                : 'border-border/60 bg-muted/30'
            }`}
          >
            {isDone ? (
              <Check className="size-4 text-emerald-500 animate-in zoom-in duration-200" />
            ) : (
              <Zap
                className={`size-4 transition-colors duration-300 ${
                  isCompiling ? 'text-primary animate-pulse' : 'text-muted-foreground/40'
                }`}
              />
            )}
          </div>
          {/* Flow lines */}
          <div className="flex h-20 w-px flex-col">
            <div
              className={`flex-1 border-l border-dashed transition-colors duration-700 ${
                isCompiling || isDone ? 'border-primary/40' : 'border-border/30'
              }`}
            />
          </div>
        </div>

        {/* Output */}
        <div className="bg-[oklch(0.12_0.01_270)] p-5">
          <p className="mb-3 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60">
            generated
          </p>
          {isCompiling && (
            <div className="flex items-center gap-2 text-[11px] text-muted-foreground animate-in fade-in duration-200">
              <Zap className="size-3 animate-pulse text-primary" />
              <span>Compiling…</span>
            </div>
          )}
          <div className="space-y-1.5">
            {OUTPUT_FILES.slice(0, outputCount).map((f, i) => (
              <div
                key={i}
                className="flex items-center gap-2.5 animate-in fade-in slide-in-from-right-2 duration-200"
              >
                <Check className="size-3 shrink-0 text-emerald-500" />
                <ProviderLogo provider={f.provider} size="sm" />
                <span className="font-mono text-[11px] text-foreground/70">{f.file}</span>
                <span className="ml-auto text-[9px] text-muted-foreground/40">{f.ms}ms</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}

// ── Provider tabs output preview ───────────────────────────────────────────

const PROVIDER_TABS = [
  { id: 'claude',  label: 'Claude Code' },
  { id: 'gemini',  label: 'Gemini CLI'  },
  { id: 'codex',   label: 'Codex CLI'   },
  { id: 'cursor',  label: 'Cursor'      },
]

const PROVIDER_OUTPUTS: Record<string, { filename: string; content: string }> = {
  claude: {
    filename: 'CLAUDE.md + .mcp.json',
    content: `# CLAUDE.md

## Rules
Use TypeScript. Prefer explicit types.
No workarounds without a linked issue.

## Skills
- commit-conventions: atomic, well-described commits
- code-review: structured review checklist

## MCP servers
- github: search repos, manage PRs
- linear: issues and project cycles
- memory: persist context across sessions`,
  },
  gemini: {
    filename: 'GEMINI.md + .gemini/settings.json',
    content: `# GEMINI.md

## Rules
Use TypeScript. Prefer explicit types.
No workarounds without a linked issue.

## Skills
- commit-conventions: atomic, well-described commits
- code-review: structured review checklist`,
  },
  codex: {
    filename: 'AGENTS.md + .codex/config.toml',
    content: `# AGENTS.md

## Rules
Use TypeScript. Prefer explicit types.
No workarounds without a linked issue.

## Skills
- commit-conventions: atomic, well-described commits
- code-review: structured review checklist`,
  },
  cursor: {
    filename: '.cursor/mcp.json + .cursor/rules/',
    content: `// .cursor/mcp.json
{
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": { "GITHUB_TOKEN": "$GITHUB_TOKEN" }
    },
    "linear": {
      "command": "npx",
      "args": ["-y", "@linear/mcp-server"],
      "env": { "LINEAR_API_KEY": "$LINEAR_API_KEY" }
    }
  }
}`,
  },
}

// ── Page ───────────────────────────────────────────────────────────────────

function HomePage() {
  const [activeProvider, setActiveProvider] = useState('claude')
  const output = PROVIDER_OUTPUTS[activeProvider]

  return (
    <main className="min-h-screen">
      {/* Hero */}
      <section className="relative overflow-hidden px-6 pb-20 pt-24 sm:px-10 sm:pt-32">
        <div className="pointer-events-none absolute -left-32 -top-32 h-96 w-96 rounded-full bg-[radial-gradient(circle,oklch(0.67_0.16_58_/_18%),transparent_66%)]" />
        <div className="pointer-events-none absolute -right-32 -top-16 h-96 w-96 rounded-full bg-[radial-gradient(circle,oklch(0.61_0.23_303_/_14%),transparent_66%)]" />

        <div className="relative mx-auto max-w-4xl text-center">
          <div className="mb-6 inline-flex items-center gap-2 rounded-full border border-primary/20 bg-primary/8 px-3 py-1.5 text-xs font-semibold tracking-wide text-primary uppercase">
            <Sparkles className="size-3" />
            Early Access
          </div>

          <h1 className="mb-6 font-display text-5xl font-bold tracking-tight sm:text-7xl">
            Your agents,{' '}
            <span className="text-primary">your rules.</span>
          </h1>

          <p className="mx-auto mb-10 max-w-2xl text-lg text-muted-foreground sm:text-xl">
            Configure MCP servers, skills, and permissions once — export to Claude Code, Gemini CLI, Codex CLI, and Cursor with a single click.
          </p>

          <div className="flex flex-wrap justify-center gap-3">
            <a
              href="/studio"
              className="inline-flex items-center gap-2 rounded-full bg-primary px-6 py-3 text-sm font-semibold text-primary-foreground transition hover:-translate-y-0.5 hover:opacity-90"
            >
              Open Studio
              <ArrowRight className="size-4" />
            </a>
            <a
              href="#how-it-works"
              className="inline-flex items-center gap-2 rounded-full border border-border bg-card px-6 py-3 text-sm font-semibold transition hover:-translate-y-0.5 hover:border-border/80"
            >
              How it works
              <ChevronDown className="size-4" />
            </a>
          </div>

          {/* Compiler animation */}
          <CompilerAnimation />
        </div>
      </section>

      {/* Provider output preview */}
      <section className="border-y border-border/60 bg-muted/30 px-6 py-12 sm:px-10">
        <div className="mx-auto max-w-5xl">
          <p className="mb-6 text-center text-xs font-semibold tracking-widest text-muted-foreground uppercase">
            Export to
          </p>
          <div className="mb-6 flex flex-wrap items-center justify-center gap-2">
            {PROVIDER_TABS.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveProvider(tab.id)}
                className={`flex items-center gap-2 rounded-full border px-4 py-2 text-sm font-medium transition ${
                  activeProvider === tab.id
                    ? 'border-primary/30 bg-primary/10 text-foreground'
                    : 'border-border/60 bg-card text-muted-foreground hover:border-border hover:text-foreground'
                }`}
              >
                <ProviderLogo provider={tab.id} size="sm" />
                {tab.label}
              </button>
            ))}
          </div>
          <div
            key={activeProvider}
            className="overflow-hidden rounded-xl border border-border/60 bg-card animate-in fade-in slide-in-from-bottom-2 duration-300"
          >
            <div className="flex items-center gap-2 border-b border-border/60 bg-muted/30 px-4 py-2.5">
              <ProviderLogo provider={activeProvider} size="sm" />
              <p className="font-mono text-[11px] font-medium text-muted-foreground">{output.filename}</p>
            </div>
            <pre className="overflow-x-auto p-5 text-xs leading-relaxed text-foreground/80">{output.content}</pre>
          </div>
        </div>
      </section>

      {/* How it works */}
      <section id="how-it-works" className="px-6 py-20 sm:px-10">
        <div className="mx-auto max-w-4xl">
          <h2 className="mb-12 text-center font-display text-3xl font-bold sm:text-4xl">
            How it works
          </h2>
          <div className="grid gap-5 sm:grid-cols-3">
            {[
              {
                icon: Layers,
                step: '01',
                title: 'Build your library',
                description: 'Add MCP servers, skills, and rules from the curated catalog or your own.',
              },
              {
                icon: Zap,
                step: '02',
                title: 'Configure your mode',
                description: 'Choose which AI agents to target. Permissions apply per provider automatically.',
              },
              {
                icon: Bot,
                step: '03',
                title: 'Export everywhere',
                description: 'Download provider-native config files. All agents start with the same context.',
              },
            ].map(({ icon: Icon, step, title, description }) => (
              <div key={step} className="rounded-2xl border border-border/60 bg-card p-5">
                <div className="mb-3 flex items-center gap-3">
                  <span className="font-display text-2xl font-bold text-primary/25">{step}</span>
                  <div className="flex size-8 items-center justify-center rounded-lg border border-primary/20 bg-primary/10">
                    <Icon className="size-4 text-primary" />
                  </div>
                </div>
                <h3 className="mb-1.5 text-sm font-semibold">{title}</h3>
                <p className="text-xs leading-relaxed text-muted-foreground">{description}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* CTA */}
      <section className="px-6 pb-24 sm:px-10">
        <div className="mx-auto max-w-2xl rounded-2xl border border-primary/20 bg-primary/5 p-10 text-center">
          <h2 className="mb-3 font-display text-2xl font-bold sm:text-3xl">
            Ready to unify your agent stack?
          </h2>
          <p className="mb-6 text-sm text-muted-foreground">
            Configure once and get provider-ready output files in seconds — entirely in your browser.
          </p>
          <a
            href="/studio"
            className="inline-flex items-center gap-2 rounded-full bg-primary px-6 py-3 text-sm font-semibold text-primary-foreground transition hover:-translate-y-0.5 hover:opacity-90"
          >
            Open Ship Studio
            <ArrowRight className="size-4" />
          </a>
        </div>
      </section>
    </main>
  )
}
