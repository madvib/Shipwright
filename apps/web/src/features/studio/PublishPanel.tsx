import { useState, useEffect } from 'react'
import { X, Copy, CheckCheck, Loader2 } from 'lucide-react'
import { ProviderLogo } from '#/features/compiler/ProviderLogo'
import { PROVIDER_SHORT } from '#/features/compiler/components/ModeHeader'
import type { CompileState } from '#/features/compiler/useCompiler'
import type { CompileResult } from '#/features/compiler/types'

interface PublishPanelProps {
  library: any
  compileState: CompileState
  onClose: () => void
}

export function PublishPanel({ library, compileState, onClose }: PublishPanelProps) {
  const hasContent = (library?.skills?.length ?? 0) > 0 || (library?.mcp_servers?.length ?? 0) > 0

  return (
    <aside className="w-96 border-l border-border/60 bg-card/30 flex flex-col overflow-hidden shrink-0">
      {/* Header */}
      <div className="px-4 py-2.5 border-b border-border/40 flex items-center justify-between shrink-0">
        <h3 className="text-xs font-semibold text-foreground flex items-center gap-2">
          Live Preview
          {compileState.status === 'ok' && (
            <span className="flex items-center gap-1 text-[10px] font-normal text-emerald-500">
              <span className="size-1.5 rounded-full bg-emerald-500 animate-pulse" />
              {compileState.elapsed}ms
            </span>
          )}
        </h3>
        <button onClick={onClose} className="text-muted-foreground/50 hover:text-muted-foreground">
          <X className="size-3.5" />
        </button>
      </div>

      {/* Output preview */}
      <OutputSection compileState={compileState} hasContent={hasContent} />
    </aside>
  )
}

const ALL_PREVIEW_PROVIDERS = ['claude', 'gemini', 'codex', 'cursor', 'opencode']

function OutputSection({ compileState, hasContent }: {
  compileState: CompileState; hasContent: boolean
}) {
  const [activeProvider, setActiveProvider] = useState('claude')
  const [activeFile, setActiveFile] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)

  const output = compileState.status === 'ok' ? compileState.output : null
  const current = output?.[activeProvider] ?? null
  const tabs = current ? getFileTabs(activeProvider, current) : []

  useEffect(() => {
    if (tabs.length > 0 && !tabs.find((t) => t.id === activeFile)) {
      setActiveFile(tabs[0].id)
    }
  }, [activeProvider, compileState.status, tabs, activeFile])

  const displayTab = tabs.find((t) => t.id === activeFile) ?? tabs[0] ?? null
  const text = displayTab?.content ?? null

  const copy = () => {
    if (!text) return
    void navigator.clipboard.writeText(text).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    })
  }

  return (
    <div className="flex-1 flex flex-col min-h-0 overflow-hidden">
      {ALL_PREVIEW_PROVIDERS.length > 0 && (
        <div className="flex items-center gap-0.5 border-b border-border/40 px-2 py-1.5 shrink-0 overflow-x-auto [scrollbar-width:none]">
          {ALL_PREVIEW_PROVIDERS.map((p) => (
            <button
              key={p}
              onClick={() => setActiveProvider(p)}
              className={`flex shrink-0 items-center gap-1.5 rounded-md px-2 py-1 text-[11px] font-medium transition ${
                activeProvider === p
                  ? 'bg-muted text-foreground'
                  : 'text-muted-foreground/60 hover:text-foreground'
              }`}
            >
              <ProviderLogo provider={p} />
              {PROVIDER_SHORT[p] ?? p}
            </button>
          ))}
        </div>
      )}

      {tabs.length > 0 && (
        <div className="flex items-center justify-between border-b border-border/40 px-2 py-1 shrink-0">
          <div className="flex items-center gap-0.5 overflow-x-auto [scrollbar-width:none]">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveFile(tab.id)}
                className={`shrink-0 rounded px-1.5 py-0.5 text-[9px] font-mono font-medium transition ${
                  activeFile === tab.id ? 'bg-muted text-foreground' : 'text-muted-foreground/50 hover:text-foreground'
                }`}
              >
                {tab.filename}
              </button>
            ))}
          </div>
          <button onClick={copy} disabled={!text} className="rounded p-1 text-muted-foreground hover:text-foreground disabled:opacity-30 transition">
            {copied ? <CheckCheck className="size-3 text-emerald-500" /> : <Copy className="size-3" />}
          </button>
        </div>
      )}

      <div className="flex-1 min-h-0 overflow-hidden">
        {compileState.status === 'idle' && (
          <div className="flex flex-col items-center justify-center h-full p-6 text-center">
            {hasContent ? (
              <p className="text-xs text-muted-foreground">Config ready to preview</p>
            ) : (
              <p className="text-xs text-muted-foreground">Select an agent to see compiled output</p>
            )}
          </div>
        )}
        {compileState.status === 'compiling' && (
          <div className="flex items-center justify-center h-full">
            <Loader2 className="size-4 animate-spin text-muted-foreground" />
          </div>
        )}
        {compileState.status === 'error' && (
          <div className="p-3 text-xs text-destructive">{compileState.message}</div>
        )}
        {compileState.status === 'ok' && text && (
          <pre className="h-full overflow-auto p-3 font-mono text-[10px] leading-relaxed text-foreground/70 whitespace-pre-wrap break-all">
            {text}
          </pre>
        )}
      </div>
    </div>
  )
}

function getFileTabs(provider: string, result: CompileResult) {
  const tabs: { id: string; filename: string; content: string }[] = []
  const ctx = result.context_content
  if (ctx) {
    const name = provider === 'claude' ? 'CLAUDE.md' : provider === 'gemini' ? 'GEMINI.md' : 'AGENTS.md'
    tabs.push({ id: 'context', filename: name, content: ctx })
  }
  if (result.mcp_servers) {
    const path = provider === 'gemini' ? '.gemini/settings.json' : provider === 'cursor' ? '.cursor/mcp.json' : '.mcp.json'
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
  if (result.cursor_hooks_patch) {
    tabs.push({ id: 'cursor-hooks', filename: '.cursor/hooks.json', content: JSON.stringify(result.cursor_hooks_patch, null, 2) })
  }
  const ruleFiles = result.rule_files ?? {}
  Object.entries(ruleFiles).forEach(([path, content]) => {
    tabs.push({ id: `rule-${path}`, filename: path, content: content ?? '' })
  })
  return tabs
}
