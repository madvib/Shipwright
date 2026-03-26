import { createFileRoute } from '@tanstack/react-router'
import { useState } from 'react'
import { Terminal, Plug, Palette, Sun, Moon, Monitor, RotateCcw } from 'lucide-react'
import { Button } from '@ship/primitives'
import { toast } from 'sonner'
import { SettingsSection, SettingsRow, SettingsSelect } from '#/features/settings/SettingsLayout'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import { SettingsSkeleton } from '#/features/studio/StudioSkeleton'

export const Route = createFileRoute('/studio/settings')({
  component: SettingsPage,
  pendingComponent: SettingsSkeleton,
  ssr: false,
})

function SettingsPage() {
  return (
    <div className="mx-auto max-w-[680px] px-5 py-6 pb-24">
      <div className="mb-6">
        <h1 className="font-display text-xl font-bold text-foreground">Settings</h1>
        <p className="text-[13px] text-muted-foreground">Studio and CLI configuration</p>
      </div>

      <CLIConnectionSection />
      <AppearanceSection />
      <LocalDataSection />
    </div>
  )
}

function CLIConnectionSection() {
  const mcp = useLocalMcpContext()
  const [portInput, setPortInput] = useState(String(mcp?.port ?? 51741))

  const handlePortSave = () => {
    const p = parseInt(portInput, 10)
    if (p > 0 && p < 65536 && mcp) mcp.setPort(p)
  }

  return (
    <SettingsSection icon={<Terminal className="size-[15px]" />} title="CLI Connection">
      <SettingsRow label="MCP port" sublabel="Port for the local Ship MCP bridge">
        <div className="flex items-center gap-2">
          <input
            type="number"
            value={portInput}
            onChange={(e) => setPortInput(e.target.value)}
            onBlur={handlePortSave}
            onKeyDown={(e) => e.key === 'Enter' && handlePortSave()}
            className="w-24 rounded-md border border-border/60 bg-muted/30 px-2 py-1 text-[11px] font-mono text-foreground outline-none focus:border-primary/50"
          />
          {mcp && (
            <span className={`flex items-center gap-1.5 text-[10px] ${
              mcp.status === 'connected' ? 'text-emerald-600 dark:text-emerald-400' : 'text-muted-foreground'
            }`}>
              <span className={`size-1.5 rounded-full ${
                mcp.status === 'connected' ? 'bg-emerald-500' :
                mcp.status === 'connecting' ? 'bg-amber-500 animate-pulse' : 'bg-muted-foreground/40'
              }`} />
              {mcp.status === 'connected' ? 'Connected' :
               mcp.status === 'connecting' ? 'Connecting...' : 'Disconnected'}
            </span>
          )}
        </div>
      </SettingsRow>
      <SettingsRow label="Install CLI" sublabel="Required for local compilation and project sync">
        <code className="font-mono text-[11px] text-muted-foreground">cargo install ship-studio-cli</code>
      </SettingsRow>
      <SettingsRow label="Start MCP bridge" sublabel="Run in your project directory">
        <code className="font-mono text-[11px] text-muted-foreground">ship mcp serve --http --port {portInput}</code>
      </SettingsRow>
    </SettingsSection>
  )
}

function AppearanceSection() {
  const [theme, setTheme] = useState<string>(() => {
    try { return localStorage.getItem('theme') ?? 'system' } catch { return 'system' }
  })

  const applyTheme = (value: string) => {
    setTheme(value)
    const resolved = value === 'system'
      ? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
      : value
    document.documentElement.classList.remove('light', 'dark')
    document.documentElement.classList.add(resolved)
    document.documentElement.setAttribute('data-theme', resolved)
    document.documentElement.style.colorScheme = resolved
    localStorage.setItem('theme', value)
  }

  return (
    <SettingsSection icon={<Palette className="size-[15px]" />} title="Appearance">
      <SettingsRow label="Theme">
        <SettingsSelect
          value={theme}
          onChange={applyTheme}
          options={[
            { value: 'light', label: 'Light' },
            { value: 'dark', label: 'Dark' },
            { value: 'system', label: 'System' },
          ]}
        />
      </SettingsRow>
    </SettingsSection>
  )
}

function LocalDataSection() {
  const handleReset = () => {
    const keys = ['ship-agents-v2', 'ship-studio-v1', 'ship-skills-ide-v1', 'ship-settings-v1']
    for (const key of keys) localStorage.removeItem(key)
    toast.success('Local data cleared — refreshing')
    setTimeout(() => window.location.reload(), 500)
  }

  return (
    <SettingsSection icon={<RotateCcw className="size-[15px]" />} title="Local Data">
      <SettingsRow
        label="Reset Studio"
        sublabel="Clear all locally stored agents, skills, and preferences. Does not affect your CLI project."
      >
        <Button variant="outline" size="xs" onClick={() => {
          if (confirm('Reset all local Studio data? This only clears the browser — your .ship/ directory is untouched.')) {
            handleReset()
          }
        }}>
          Reset
        </Button>
      </SettingsRow>
    </SettingsSection>
  )
}
