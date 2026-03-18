import { useState } from 'react'
import { Upload, Download, PanelLeft, Loader2 } from 'lucide-react'
import { ProviderLogo } from '#/features/compiler/ProviderLogo'
import type { CompileState } from '#/features/compiler/useCompiler'
import type { ProjectLibrary } from '#/features/compiler/types'

export const PROVIDER_SHORT: Record<string, string> = {
  claude: 'Claude',
  gemini: 'Gemini',
  codex: 'Codex',
  cursor: 'Cursor',
}

function triggerDownload(text: string, name: string) {
  const blob = new Blob([text], { type: 'text/plain' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = name
  a.click()
  URL.revokeObjectURL(url)
}

export { triggerDownload }

interface ExportButtonProps {
  state: CompileState
  selectedProviders: string[]
  getInspectorTabs: (provider: string) => Array<{ id: string; filename: string; content: string }>
}

function ExportButton({ state, selectedProviders, getInspectorTabs }: ExportButtonProps) {
  const [open, setOpen] = useState(false)
  const output = state.status === 'ok' ? state.output : null

  const downloadProvider = (p: string) => {
    getInspectorTabs(p).forEach((tab) => triggerDownload(tab.content, tab.filename))
    setOpen(false)
  }

  const downloadAll = () => {
    selectedProviders.forEach((p) => downloadProvider(p))
    setOpen(false)
  }

  return (
    <div className="relative">
      <button
        onClick={() => (output ? setOpen((v) => !v) : undefined)}
        disabled={!output}
        className="inline-flex items-center gap-1.5 rounded-lg bg-primary px-3 py-1.5 text-xs font-semibold text-primary-foreground transition hover:opacity-90 disabled:opacity-40"
      >
        <Download className="size-3" />
        Export
      </button>
      {open && output && (
        <>
          <div className="fixed inset-0 z-10" onClick={() => setOpen(false)} />
          <div className="absolute right-0 top-full z-20 mt-1 w-44 rounded-xl border border-border/60 bg-card shadow-lg overflow-hidden">
            {selectedProviders.map((p) =>
              output[p] ? (
                <button
                  key={p}
                  onClick={() => downloadProvider(p)}
                  className="flex w-full items-center gap-2 px-3 py-2 text-xs hover:bg-muted transition text-left"
                >
                  <ProviderLogo provider={p} />
                  {PROVIDER_SHORT[p] ?? p}
                </button>
              ) : null
            )}
            <div className="border-t border-border/60" />
            <button
              onClick={downloadAll}
              className="flex w-full items-center gap-2 px-3 py-2 text-xs font-medium hover:bg-muted transition text-left"
            >
              <Download className="size-3" />
              All providers
            </button>
          </div>
        </>
      )}
    </div>
  )
}

export interface ModeHeaderProps {
  modeName: string
  onModeNameChange: (name: string) => void
  library: ProjectLibrary
  state: CompileState
  selectedProviders: string[]
  showLibrary: boolean
  onToggleLibrary: () => void
  onOpenImport: () => void
  getInspectorTabs: (provider: string) => Array<{ id: string; filename: string; content: string }>
}

export function ModeHeader({
  modeName,
  onModeNameChange,
  library,
  state,
  selectedProviders,
  showLibrary,
  onToggleLibrary,
  onOpenImport,
  getInspectorTabs,
}: ModeHeaderProps) {
  const isGenerating = state.status === 'compiling'
  const mcpCount = library.mcp_servers.length
  const skillCount = library.skills.length
  const ruleCount = library.rules.length

  return (
    <div className="flex items-center gap-2 border-b border-border/60 bg-card/50 px-3 py-2 shrink-0 backdrop-blur-sm">
      <button
        onClick={onToggleLibrary}
        title={showLibrary ? 'Hide library' : 'Show library'}
        className={`flex size-7 items-center justify-center rounded-md transition hover:bg-muted ${showLibrary ? 'text-foreground' : 'text-muted-foreground'}`}
      >
        <PanelLeft className="size-3.5" />
      </button>
      <div className="h-4 w-px bg-border/60" />
      <input
        value={modeName}
        onChange={(e) => onModeNameChange(e.target.value)}
        className="min-w-0 rounded px-1.5 py-0.5 font-display text-sm font-semibold text-foreground bg-transparent border border-transparent focus:border-border/60 focus:bg-card focus:outline-none transition w-40"
        placeholder="untitled-mode"
        spellCheck={false}
      />
      <div className="hidden sm:flex items-center gap-1.5 ml-1">
        {mcpCount > 0 && (
          <span className="rounded-full bg-primary/10 px-2 py-0.5 text-[10px] font-semibold text-primary">{mcpCount} MCP</span>
        )}
        {skillCount > 0 && (
          <span className="rounded-full bg-cyan-500/10 px-2 py-0.5 text-[10px] font-semibold text-cyan-600 dark:text-cyan-400">{skillCount} skills</span>
        )}
        {ruleCount > 0 && (
          <span className="rounded-full bg-amber-500/10 px-2 py-0.5 text-[10px] font-semibold text-amber-600 dark:text-amber-400">{ruleCount} rules</span>
        )}
      </div>
      <div className="ml-auto flex items-center gap-2">
        {isGenerating && (
          <div className="flex items-center gap-1.5 text-[10px] text-muted-foreground">
            <Loader2 className="size-3 animate-spin" />
            <span className="hidden sm:inline">Generating…</span>
          </div>
        )}
        {state.status === 'ok' && (
          <span className="hidden sm:inline text-[10px] text-muted-foreground">{state.elapsed}ms · WASM</span>
        )}
        <button
          onClick={onOpenImport}
          className="inline-flex items-center gap-1.5 rounded-lg border border-border/60 bg-card px-3 py-1.5 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground"
        >
          <Upload className="size-3" />
          <span className="hidden sm:inline">Import</span>
        </button>
        <ExportButton state={state} selectedProviders={selectedProviders} getInspectorTabs={getInspectorTabs} />
      </div>
    </div>
  )
}
