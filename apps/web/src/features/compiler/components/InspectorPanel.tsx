import { useState, useEffect } from 'react'
import { Copy, CheckCheck, Download, Zap, Loader2 } from 'lucide-react'
import { toast } from 'sonner'
import { ProviderLogo } from '#/features/compiler/ProviderLogo'
import { PROVIDER_SHORT, triggerDownload } from '#/features/compiler/components/ModeHeader'
import type { CompileState } from '#/features/compiler/useCompiler'
import type { CompileResult } from '#/features/compiler/types'

export interface InspectorTab {
  id: string
  filename: string
  content: string
}

export function getInspectorTabs(provider: string, result: CompileResult): InspectorTab[] {
  const tabs: InspectorTab[] = []

  const ctx = result.context_content
  if (ctx) {
    const name =
      provider === 'claude' ? 'CLAUDE.md'
      : provider === 'gemini' ? 'GEMINI.md'
      : provider === 'codex' ? 'AGENTS.md'
      : 'AGENTS.md'
    tabs.push({ id: 'context', filename: name, content: ctx })
  }

  if (result.mcp_servers) {
    const path =
      provider === 'gemini' ? '.gemini/settings.json'
      : provider === 'cursor' ? '.cursor/mcp.json'
      : '.mcp.json'
    tabs.push({ id: 'mcp', filename: path, content: JSON.stringify(result.mcp_servers, null, 2) })
  }

  if (result.claude_settings_patch) {
    tabs.push({ id: 'claude-settings', filename: '.claude/settings.json', content: JSON.stringify(result.claude_settings_patch, null, 2) })
  }

  if (result.codex_config_patch) {
    tabs.push({ id: 'codex-config', filename: '.codex/config.toml', content: result.codex_config_patch })
  }

  if (result.gemini_settings_patch && provider === 'gemini') {
    tabs.push({ id: 'gemini-settings', filename: '.gemini/settings.json', content: JSON.stringify(result.gemini_settings_patch, null, 2) })
  }

  if (result.gemini_policy_patch) {
    tabs.push({ id: 'gemini-policy', filename: '.gemini/policies/ship.toml', content: result.gemini_policy_patch })
  }

  if (Object.keys(result.rule_files ?? {}).length > 0 && provider === 'cursor') {
    Object.entries(result.rule_files ?? {}).forEach(([path, content]) => {
      tabs.push({ id: `rule-${path}`, filename: path, content })
    })
  }

  return tabs
}

export interface InspectorPanelProps {
  state: CompileState
  selectedProviders: string[]
}

export function InspectorPanel({ state, selectedProviders }: InspectorPanelProps) {
  const [activeProvider, setActiveProvider] = useState(selectedProviders[0] ?? 'claude')
  const [activeFile, setActiveFile] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)

  useEffect(() => {
    if (!selectedProviders.includes(activeProvider) && selectedProviders.length > 0) {
      setActiveProvider(selectedProviders[0])
    }
  }, [selectedProviders, activeProvider])

  const output = state.status === 'ok' ? state.output : null
  const current = output?.[activeProvider] ?? null
  const tabs = current ? getInspectorTabs(activeProvider, current) : []

  useEffect(() => {
    if (tabs.length > 0) setActiveFile(tabs[0].id)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeProvider, state.status])

  const displayTab = tabs.find((t) => t.id === activeFile) ?? tabs[0] ?? null
  const text = displayTab?.content ?? null

  const copy = () => {
    if (!text) return
    void navigator.clipboard.writeText(text).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
      toast.success('Copied to clipboard')
    })
  }

  return (
    <aside className="hidden md:flex w-96 xl:w-[420px] shrink-0 flex-col bg-sidebar/20">
      <div className="flex items-center gap-0.5 border-b border-border/60 bg-muted/20 px-2 py-1.5 shrink-0 overflow-x-auto [scrollbar-width:none]">
        {selectedProviders.map((p) => (
          <button
            key={p}
            onClick={() => setActiveProvider(p)}
            className={`flex shrink-0 items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs font-medium transition ${
              activeProvider === p
                ? 'bg-card text-foreground shadow-sm'
                : 'text-muted-foreground hover:bg-muted/60 hover:text-foreground'
            }`}
          >
            <ProviderLogo provider={p} />
            {PROVIDER_SHORT[p] ?? p}
          </button>
        ))}
      </div>

      {tabs.length > 0 && (
        <div className="flex items-center justify-between border-b border-border/60 px-2 py-1 shrink-0">
          <div className="flex items-center gap-0.5 overflow-x-auto [scrollbar-width:none]">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveFile(tab.id)}
                className={`shrink-0 rounded px-2 py-1 text-[10px] font-mono font-medium transition ${
                  activeFile === tab.id
                    ? 'bg-card text-foreground shadow-sm'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
              >
                {tab.filename}
              </button>
            ))}
          </div>
          <div className="flex items-center gap-0.5 shrink-0">
            <button
              onClick={copy}
              disabled={!text}
              className="rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground disabled:opacity-40 transition"
              title="Copy"
            >
              {copied ? <CheckCheck className="size-3 text-emerald-500" /> : <Copy className="size-3" />}
            </button>
            <button
              onClick={() => { if (text && displayTab) triggerDownload(text, displayTab.filename) }}
              disabled={!text}
              className="rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground disabled:opacity-40 transition"
              title="Download file"
            >
              <Download className="size-3" />
            </button>
          </div>
        </div>
      )}

      <div className="flex flex-1 min-h-0 flex-col overflow-hidden">
        {state.status === 'idle' && (
          <div className="flex flex-1 flex-col items-center justify-center gap-3 p-6 text-center">
            <div className="flex size-12 items-center justify-center rounded-xl border border-border/60 bg-muted/40">
              <Zap className="size-4 text-muted-foreground" />
            </div>
            <div>
              <p className="text-xs font-medium text-foreground">Your config will appear here</p>
              <p className="mt-1 text-[11px] text-muted-foreground">Add an MCP server or skill from the library</p>
            </div>
          </div>
        )}
        {state.status === 'compiling' && (
          <div className="flex flex-1 items-center justify-center">
            <div className="flex items-center gap-2 text-[11px] text-muted-foreground">
              <Loader2 className="size-3.5 animate-spin" />
              Generating…
            </div>
          </div>
        )}
        {state.status === 'error' && (
          <div className="p-4">
            <p className="text-xs font-medium text-destructive mb-2">Generation failed</p>
            <pre className="text-[10px] text-destructive/80 leading-relaxed whitespace-pre-wrap">{state.message}</pre>
          </div>
        )}
        {state.status === 'ok' && (
          <>
            {text ? (
              <div className="flex-1 overflow-auto">
                <pre className="p-4 font-mono text-[11px] leading-relaxed text-foreground/80 whitespace-pre-wrap break-all">
                  {text}
                </pre>
              </div>
            ) : (
              <div className="flex flex-1 items-center justify-center">
                <p className="text-[11px] text-muted-foreground">No output for this file.</p>
              </div>
            )}
          </>
        )}
      </div>

      {state.status === 'ok' && (
        <div className="shrink-0 border-t border-border/60 bg-muted/20 px-3 py-2 flex items-center justify-between">
          <p className="text-[10px] text-muted-foreground">Generated in {state.elapsed}ms · WASM</p>
          <button
            onClick={() => tabs.forEach((tab) => triggerDownload(tab.content, tab.filename))}
            className="inline-flex items-center gap-1 rounded-md bg-primary px-2.5 py-1 text-[10px] font-semibold text-primary-foreground transition hover:opacity-90"
          >
            <Download className="size-2.5" />
            Export
          </button>
        </div>
      )}
    </aside>
  )
}
