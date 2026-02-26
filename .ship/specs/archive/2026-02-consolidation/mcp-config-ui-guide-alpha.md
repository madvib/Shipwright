# Shipwright — Agent Config UI: Alpha Build Guide

**Version:** 0.1  
**Scope:** Alpha — MCP server management UI for Claude Code, Gemini CLI, Codex  
**Last Updated:** 2026-02-22

---

## What We're Building

The Agent tab in Settings today is a text input with no intelligence. We're replacing it with a first-class MCP management experience: auto-discovery, import, visual server management, mode assignment, and one-click propagation to all managed AI CLIs.

This is potentially the most important surface in the entire application for first impressions. A developer who opens this tab and sees their existing MCP servers already discovered and organized will convert immediately.

---

## Information Architecture

### Where It Lives

The current settings page (Project / Global / Agent tabs) gets a promoted Agent tab that becomes its own dedicated page at a certain complexity threshold. For alpha, keep it as a tab but design it as if it's a full page — generous layout, room to breathe, no cramming.

```
Settings
├── Project          (name, git strategy, templates)
├── Global           (auth, appearance, defaults)
└── Agents           ← This guide
    ├── Overview     (detected tools, active mode, quick status)
    ├── Servers      (all defined servers, manage/add/edit)
    ├── Modes        (mode definitions, server assignment)
    └── Tools        (managed AI CLI tools)
```

The four sub-sections can be a secondary tab row, or a single scrollable page with anchored sections. **Recommendation: single scrollable page for alpha** — tab proliferation is confusing and the content is cohesive. Use sticky section headers to maintain orientation.

---

## The Complete UX Flow

### First Open (No Existing Config)

```
┌─────────────────────────────────────────────────────────────────┐
│  Agents                                                          │
│                                                                  │
│  ╔═══════════════════════════════════════════════════════════╗  │
│  ║  🔍  Let's find your existing setup                       ║  │
│  ║                                                           ║  │
│  ║  Shipwright can import MCP servers you've already         ║  │
│  ║  configured in Claude Code, Gemini CLI, or Codex.         ║  │
│  ║                                                           ║  │
│  ║  [Scan for existing configs]    [Start from scratch]      ║  │
│  ╚═══════════════════════════════════════════════════════════╝  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

This is the only moment an onboarding prompt is shown. After first interaction it disappears.

### First Open (After Scan)

The scan result is shown immediately — no loading screen longer than 500ms. If scanning takes longer, show progress inline.

```
┌─────────────────────────────────────────────────────────────────┐
│  Agents                                              [+ Add Server] │
│                                                                  │
│  ACTIVE MODE  ●  Execution                     [Switch ▾]       │
│  ─────────────────────────────────────────────────────────────  │
│                                                                  │
│  AI TOOLS  ───────────────────────────────────────────────────  │
│                                                                  │
│  ✅  Claude Code    Managed    .mcp.json                        │
│  ✅  Gemini CLI     Managed    .gemini/settings.json            │
│  ⚪  Codex          Detected   not managed      [Manage]        │
│                                                                  │
│  MCP SERVERS  ─────────────────────────────────────────────────  │
│                                                                  │
│  ● shipwright    always active    stdio                         │
│  ● github        execution        npx github-mcp      [Edit]   │
│  ● postgres      backend          npx postgres-mcp    [Edit]   │
│  ○ figma         planning         npx figma-mcp       [Edit]   │
│                                                                  │
│  [+ Add Server]                                                 │
│                                                                  │
│  MODES  ───────────────────────────────────────────────────────  │
│                                                                  │
│  ● Planning     figma, linear, shipwright         [Edit]        │
│  ● Execution    github, shipwright                [Edit] [●]    │
│  ○ Backend      postgres, shipwright              [Edit]        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Component Breakdown

### 1. Active Mode Bar

Persistent strip at the top of the Agents page. Always visible. Switching mode here triggers the full write pipeline across all managed tools.

```tsx
// components/agents/ActiveModeBar.tsx

interface ActiveModeBarProps {
  activeMode: Mode
  modes: Mode[]
  onSwitch: (modeId: string) => void
  switching: boolean
}

export function ActiveModeBar({ activeMode, modes, onSwitch, switching }: ActiveModeBarProps) {
  return (
    <div className="active-mode-bar">
      <div className="mode-indicator">
        <span className="mode-dot active" />
        <span className="mode-label">Active Mode</span>
        <span className="mode-name">{activeMode.name}</span>
      </div>

      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <button className="mode-switch-btn" disabled={switching}>
            {switching ? <Spinner size="sm" /> : 'Switch'}
            <ChevronDown size={14} />
          </button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          {modes.map(mode => (
            <DropdownMenuItem
              key={mode.id}
              onSelect={() => onSwitch(mode.id)}
              className={mode.id === activeMode.id ? 'active' : ''}
            >
              <span className="mode-dot" style={{ background: mode.color }} />
              {mode.name}
              <span className="mode-server-count">
                {mode.mcpServers.length} servers
              </span>
            </DropdownMenuItem>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  )
}
```

**Mode switch feedback** — switching is not instant (writes to disk, may require restart). Show inline progress, not a modal:

```
Switching to Backend mode...
  ✓ Claude Code updated
  ✓ Gemini CLI updated
  ⟳ Codex...
  
⚠ Restart Claude Code and Gemini CLI to apply changes
```

This appears as a transient notification bar below the mode bar, auto-dismisses after 6 seconds unless there's a restart warning (which stays until dismissed).

---

### 2. AI Tools Section

Shows detected tools, managed state, and config file path. The minimal version — detailed config is in a slide-over panel.

```tsx
// components/agents/AiToolsSection.tsx

export function AiToolsSection({ tools }: { tools: DetectedTool[] }) {
  return (
    <Section title="AI Tools">
      <div className="tool-list">
        {tools.map(tool => (
          <ToolRow key={tool.id} tool={tool} />
        ))}
      </div>
    </Section>
  )
}

function ToolRow({ tool }: { tool: DetectedTool }) {
  const [open, setOpen] = useState(false)

  return (
    <>
      <div className="tool-row" onClick={() => setOpen(true)}>
        <StatusDot status={tool.managedState} />
        <span className="tool-name">{tool.name}</span>
        <span className="tool-state-label">{stateLabel(tool.managedState)}</span>
        <span className="tool-config-path">{tool.configPath}</span>

        {tool.managedState === 'detected' && (
          <button
            className="btn-sm btn-primary"
            onClick={e => { e.stopPropagation(); handleManage(tool) }}
          >
            Manage
          </button>
        )}

        {tool.managedState === 'managed' && (
          <button className="btn-sm btn-ghost">
            <ChevronRight size={14} />
          </button>
        )}
      </div>

      <ToolDetailPanel tool={tool} open={open} onClose={() => setOpen(false)} />
    </>
  )
}
```

**Status dot states:**
- `●` green — managed and verified
- `●` yellow — managed, restart required
- `●` red — managed but verification failed
- `○` gray — detected, not managed
- `✕` — not found

---

### 3. Tool Detail Panel (Slide-over)

Opens from the right when clicking a tool row. Shows the raw config, individual server status, backup path, and the option to stop managing or switch to symlink mode.

```tsx
// components/agents/ToolDetailPanel.tsx

export function ToolDetailPanel({ tool, open, onClose }: ToolDetailPanelProps) {
  return (
    <SlideOver open={open} onClose={onClose} title={tool.name}>

      <Section title="Configuration">
        <DataRow label="Config path" value={tool.configPath} copyable />
        <DataRow label="Backup" value={tool.backupPath ?? 'No backup yet'} />
        <DataRow label="Last written" value={tool.lastWritten
          ? formatRelative(tool.lastWritten)
          : 'Never'
        } />
      </Section>

      <Section title="MCP Servers in this tool">
        {tool.servers.map(server => (
          <div key={server.id} className="server-status-row">
            <StatusDot status={server.verified ? 'ok' : 'error'} />
            <span>{server.id}</span>
            <span className="server-source">
              {server.shipwrightManaged ? 'Shipwright' : 'User-defined'}
            </span>
          </div>
        ))}
      </Section>

      <Section title="Raw config" collapsible defaultCollapsed>
        <CodeBlock
          language={tool.configFormat}  // "json" or "toml"
          content={tool.rawConfig}
          readOnly
        />
      </Section>

      <Section title="Advanced">
        <ToggleRow
          label="Symlink mode"
          description="Replace config file with a symlink to Shipwright's canonical file. Linux/macOS only."
          value={tool.symlinkMode}
          onChange={val => handleSymlinkToggle(tool, val)}
          warning="Some tools may not support symlinked configs. Read docs before enabling."
        />
      </Section>

      <footer className="panel-footer">
        <button
          className="btn btn-ghost btn-danger"
          onClick={() => handleUnmanage(tool)}
        >
          Stop managing
        </button>
        <button
          className="btn btn-secondary"
          onClick={() => handleVerify(tool)}
        >
          Verify config
        </button>
      </footer>

    </SlideOver>
  )
}
```

---

### 4. MCP Servers Section

The heart of the feature. Every defined server, which modes it belongs to, edit and delete inline.

```tsx
// components/agents/McpServersSection.tsx

export function McpServersSection({ servers, modes }: McpServersSectionProps) {
  const [adding, setAdding] = useState(false)

  return (
    <Section
      title="MCP Servers"
      action={
        <button className="btn-sm btn-primary" onClick={() => setAdding(true)}>
          + Add Server
        </button>
      }
    >
      {/* Shipwright always first, always locked */}
      <ServerRow
        server={servers.find(s => s.id === 'shipwright')!}
        modes={modes}
        locked
        alwaysActive
      />

      <div className="server-divider" />

      {servers
        .filter(s => s.id !== 'shipwright')
        .map(server => (
          <ServerRow
            key={server.id}
            server={server}
            modes={modes}
            onEdit={handleEdit}
            onDelete={handleDelete}
          />
        ))}

      {adding && (
        <AddServerForm
          modes={modes}
          onAdd={handleAdd}
          onCancel={() => setAdding(false)}
        />
      )}

      {servers.filter(s => s.id !== 'shipwright').length === 0 && !adding && (
        <EmptyState
          icon={<Server size={24} />}
          title="No servers defined"
          description="Add an MCP server or import from your existing AI tool configs."
          action={
            <button className="btn btn-secondary" onClick={handleImport}>
              Import existing
            </button>
          }
        />
      )}
    </Section>
  )
}
```

**Server row design:**

```tsx
function ServerRow({ server, modes, locked, alwaysActive, onEdit, onDelete }: ServerRowProps) {
  const serverModes = modes.filter(m =>
    m.mcpServers.includes(server.id)
  )

  return (
    <div className={`server-row ${locked ? 'locked' : ''}`}>

      {/* Status indicator */}
      <div className="server-status">
        <span className={`dot ${alwaysActive ? 'always' : 'conditional'}`} />
      </div>

      {/* Identity */}
      <div className="server-identity">
        <span className="server-id">{server.id}</span>
        <span className="server-transport">
          {server.transport.type === 'stdio'
            ? server.transport.command
            : server.transport.url
          }
        </span>
      </div>

      {/* Mode badges */}
      <div className="server-modes">
        {alwaysActive
          ? <Badge variant="always">always active</Badge>
          : serverModes.length === 0
            ? <Badge variant="warning">no modes</Badge>
            : serverModes.map(m => (
                <Badge key={m.id} style={{ background: m.color }}>
                  {m.name}
                </Badge>
              ))
        }
      </div>

      {/* Actions */}
      {!locked && (
        <div className="server-actions">
          <button className="icon-btn" onClick={() => onEdit(server)}>
            <Pencil size={14} />
          </button>
          <button className="icon-btn danger" onClick={() => onDelete(server.id)}>
            <Trash2 size={14} />
          </button>
        </div>
      )}
    </div>
  )
}
```

---

### 5. Add / Edit Server Form

The form that appears inline when adding or editing a server. Not a modal — inline expansion keeps spatial context.

```tsx
// components/agents/ServerForm.tsx

export function ServerForm({ server, modes, onSave, onCancel }: ServerFormProps) {
  const [transport, setTransport] = useState<'stdio' | 'http'>(
    server?.transport.type ?? 'stdio'
  )
  const [id, setId] = useState(server?.id ?? '')
  const [command, setCommand] = useState(
    server?.transport.type === 'stdio' ? server.transport.command : ''
  )
  const [args, setArgs] = useState<string[]>(
    server?.transport.type === 'stdio' ? server.transport.args : []
  )
  const [url, setUrl] = useState(
    server?.transport.type === 'http' ? server.transport.url : ''
  )
  const [env, setEnv] = useState<Record<string, string>>(
    server?.transport.type === 'stdio' ? server.transport.env : {}
  )
  const [assignedModes, setAssignedModes] = useState<string[]>(
    modes.filter(m => m.mcpServers.includes(server?.id ?? '')).map(m => m.id)
  )

  return (
    <div className="server-form">

      {/* Server ID */}
      <FormField label="Server ID" required>
        <input
          value={id}
          onChange={e => setId(e.target.value)}
          placeholder="github"
          disabled={!!server}  // can't change ID on edit
        />
        <FieldHint>Lowercase, no spaces. Used in config files and mode references.</FieldHint>
      </FormField>

      {/* Transport type */}
      <FormField label="Transport">
        <SegmentedControl
          options={[
            { value: 'stdio', label: 'stdio (command)' },
            { value: 'http', label: 'HTTP' },
          ]}
          value={transport}
          onChange={setTransport}
        />
      </FormField>

      {transport === 'stdio' ? (
        <>
          <FormField label="Command" required>
            <input
              value={command}
              onChange={e => setCommand(e.target.value)}
              placeholder="npx"
            />
          </FormField>

          <FormField label="Args">
            <ArgsInput value={args} onChange={setArgs} placeholder="-y @modelcontextprotocol/server-github" />
            <FieldHint>One argument per line, or comma-separated.</FieldHint>
          </FormField>

          <FormField label="Environment variables">
            <EnvVarsInput value={env} onChange={setEnv} />
            <FieldHint>
              Use <code>$VAR_NAME</code> to reference shell env vars.
              {' '}
              <GeminiWarning show={isGeminiManaged} />
            </FieldHint>
          </FormField>
        </>
      ) : (
        <>
          <FormField label="URL" required>
            <input
              value={url}
              onChange={e => setUrl(e.target.value)}
              placeholder="https://api.example.com/mcp"
            />
          </FormField>

          <FormField label="Bearer token env var">
            <input placeholder="MY_API_TOKEN" />
            <FieldHint>
              The name of the env var containing your token — not the token itself.
            </FieldHint>
          </FormField>
        </>
      )}

      {/* Mode assignment */}
      <FormField label="Active in modes">
        <ModeSelector
          modes={modes}
          value={assignedModes}
          onChange={setAssignedModes}
        />
        {assignedModes.length === 0 && (
          <FieldWarning>
            This server won't be active in any mode. Assign it to at least one mode.
          </FieldWarning>
        )}
      </FormField>

      {/* Suggestions */}
      <ServerSuggestions
        serverId={id}
        command={command}
        onApplySuggestion={handleSuggestion}
      />

      <div className="form-actions">
        <button className="btn btn-ghost" onClick={onCancel}>Cancel</button>
        <button
          className="btn btn-primary"
          onClick={() => handleSave()}
          disabled={!isValid()}
        >
          {server ? 'Save changes' : 'Add server'}
        </button>
      </div>

    </div>
  )
}
```

**The ArgsInput component** deserves special attention. Args are currently the biggest source of user error in MCP configs. Don't use a single text field — use a tag-style input where each arg is a pill:

```tsx
function ArgsInput({ value, onChange }: ArgsInputProps) {
  // Pills for existing args, text input to add new ones
  // Backspace on empty input removes last pill
  // Paste of "npx -y @modelcontextprotocol/server-github" auto-splits on spaces
  // Enter or comma commits current input as a new arg
  return (
    <div className="args-input">
      {value.map((arg, i) => (
        <span key={i} className="arg-pill">
          {arg}
          <button onClick={() => removeArg(i)}>×</button>
        </span>
      ))}
      <input
        placeholder={value.length === 0 ? "-y @modelcontextprotocol/server-github" : ""}
        onKeyDown={handleKeyDown}
        onPaste={handlePaste}
      />
    </div>
  )
}
```

**EnvVarsInput** — key-value pairs, not a single text field:

```tsx
function EnvVarsInput({ value, onChange }: EnvVarsInputProps) {
  return (
    <div className="env-vars-input">
      {Object.entries(value).map(([k, v]) => (
        <div key={k} className="env-row">
          <input className="env-key" value={k} onChange={...} placeholder="GITHUB_TOKEN" />
          <span>=</span>
          <input className="env-val" value={v} onChange={...} placeholder="$GITHUB_TOKEN" />
          <button onClick={() => removeEnvVar(k)}>×</button>
        </div>
      ))}
      <button className="btn-ghost btn-sm" onClick={addEnvVar}>
        + Add variable
      </button>
    </div>
  )
}
```

---

### 6. Server Suggestions

When a user types a server ID or command, Shipwright suggests known servers from a built-in registry. This is the discovery feature — no internet required, just a JSON bundle of well-known MCP servers.

```tsx
// components/agents/ServerSuggestions.tsx

// Built-in registry — bundled at compile time, updated with releases
const KNOWN_SERVERS = [
  {
    id: "github",
    name: "GitHub",
    description: "Issues, PRs, repos, gists",
    command: "npx",
    args: ["-y", "@modelcontextprotocol/server-github"],
    env: { GITHUB_PERSONAL_ACCESS_TOKEN: "$GITHUB_TOKEN" },
    suggestedModes: ["execution"],
    tags: ["vcs", "issues", "prs"],
    docsUrl: "https://github.com/modelcontextprotocol/servers",
  },
  {
    id: "postgres",
    name: "PostgreSQL",
    description: "Query, describe, and manage Postgres databases",
    command: "npx",
    args: ["-y", "@modelcontextprotocol/server-postgres"],
    env: { DATABASE_URL: "$DATABASE_URL" },
    suggestedModes: ["backend"],
    tags: ["database", "sql"],
  },
  {
    id: "figma",
    name: "Figma",
    description: "Read Figma files, components, and styles",
    command: "npx",
    args: ["-y", "figma-mcp"],
    env: { FIGMA_ACCESS_TOKEN: "$FIGMA_TOKEN" },
    suggestedModes: ["planning", "frontend"],
    tags: ["design", "ui"],
  },
  {
    id: "linear",
    name: "Linear",
    description: "Issues, projects, teams, and cycles",
    command: "npx",
    args: ["-y", "@linear/mcp-server"],
    env: { LINEAR_API_KEY: "$LINEAR_API_KEY" },
    suggestedModes: ["planning", "execution"],
    tags: ["issues", "pm"],
  },
  {
    id: "filesystem",
    name: "Filesystem",
    description: "Read and write files with path restrictions",
    command: "npx",
    args: ["-y", "@modelcontextprotocol/server-filesystem"],
    suggestedModes: ["all"],
    tags: ["files"],
  },
  // ...more
]

export function ServerSuggestions({ serverId, command, onApplySuggestion }: SuggestionsProps) {
  const match = KNOWN_SERVERS.find(s =>
    s.id.startsWith(serverId.toLowerCase()) ||
    (command && s.command === command)
  )

  if (!match || serverId === match.id) return null

  return (
    <div className="suggestion-card">
      <div className="suggestion-header">
        <Lightbulb size={14} />
        <span>Known server found</span>
      </div>
      <div className="suggestion-body">
        <strong>{match.name}</strong> — {match.description}
      </div>
      <button
        className="btn-sm btn-secondary"
        onClick={() => onApplySuggestion(match)}
      >
        Use suggested config
      </button>
    </div>
  )
}
```

**The suggestions card appears inline below the form as the user types.** It never blocks or interrupts — it's an offer, not a gate.

---

### 7. Modes Section

Visual mode editor. Each mode shows its server list and the mode switcher dot.

```tsx
// components/agents/ModesSection.tsx

export function ModesSection({ modes, servers, activeMode, onUpdate }: ModesSectionProps) {
  return (
    <Section
      title="Modes"
      action={
        <button className="btn-sm btn-ghost" onClick={handleAddMode}>
          + Add Mode
        </button>
      }
    >
      {modes.map(mode => (
        <ModeRow
          key={mode.id}
          mode={mode}
          servers={servers}
          isActive={mode.id === activeMode}
          onEdit={handleEdit}
          onActivate={() => handleSwitch(mode.id)}
        />
      ))}
    </Section>
  )
}

function ModeRow({ mode, servers, isActive, onEdit, onActivate }: ModeRowProps) {
  const modeServers = servers.filter(s =>
    mode.mcpServers.includes(s.id) || s.id === 'shipwright'
  )

  return (
    <div className={`mode-row ${isActive ? 'active' : ''}`}>

      <div className="mode-identity">
        <button
          className="mode-activate-btn"
          onClick={onActivate}
          title={isActive ? 'Currently active' : 'Switch to this mode'}
        >
          <span className={`mode-dot ${isActive ? 'active' : ''}`} />
        </button>
        <span className="mode-name">{mode.name}</span>
      </div>

      <div className="mode-servers">
        {modeServers.map(s => (
          <span key={s.id} className="server-chip">
            {s.id}
            {s.id === 'shipwright' && <Lock size={10} />}
          </span>
        ))}
        {modeServers.length === 0 && (
          <span className="no-servers-hint">No servers — add servers to this mode</span>
        )}
      </div>

      <div className="mode-actions">
        {isActive && <Badge variant="active">Active</Badge>}
        <button className="icon-btn" onClick={() => onEdit(mode)}>
          <Pencil size={14} />
        </button>
      </div>

    </div>
  )
}
```

---

### 8. Import Flow

Triggered from the empty state, a button in the header, or the onboarding card. Opens as a full-width inline panel — not a modal — because the content needs space.

```tsx
// components/agents/ImportPanel.tsx

export function ImportPanel({ onComplete, onDismiss }: ImportPanelProps) {
  const [stage, setStage] = useState<'scanning' | 'review' | 'conflict' | 'done'>('scanning')
  const [scanResult, setScanResult] = useState<ImportScanResult | null>(null)
  const [selected, setSelected] = useState<Set<string>>(new Set())
  const [modeAssignments, setModeAssignments] = useState<Record<string, string[]>>({})

  return (
    <div className="import-panel">
      {stage === 'scanning' && <ScanningState />}
      {stage === 'review' && (
        <ReviewState
          result={scanResult!}
          selected={selected}
          modeAssignments={modeAssignments}
          onToggle={handleToggle}
          onAssignMode={handleAssignMode}
          onNext={() => setStage(hasConflicts ? 'conflict' : 'done')}
          onCancel={onDismiss}
        />
      )}
      {stage === 'conflict' && (
        <ConflictState
          conflicts={scanResult!.servers.filter(s => s.conflict)}
          onResolve={handleResolve}
          onNext={() => setStage('done')}
        />
      )}
      {stage === 'done' && (
        <DoneState
          imported={importedServers}
          onComplete={onComplete}
        />
      )}
    </div>
  )
}
```

**Scanning state** — don't show a spinner and nothing else. Show what's being scanned:

```
Scanning for existing MCP configurations...

  ✓  Claude Code       3 servers found
  ✓  Gemini CLI        checking...
  ○  Codex             not detected
```

**Review state** — the most important screen in the import flow:

```
Found 5 servers across 2 tools

┌─────────────────────────────────────────────────────┐
│ ☑  github                                           │
│    npx @modelcontextprotocol/server-github          │
│    Found in: Claude Code, Gemini CLI                │
│    Assign to mode: [Execution ▾]                    │
├─────────────────────────────────────────────────────┤
│ ☑  postgres                                         │
│    npx @modelcontextprotocol/server-postgres        │
│    Found in: Gemini CLI                             │
│    Assign to mode: [Backend ▾]                      │
├─────────────────────────────────────────────────────┤
│ ☑  figma                                            │
│    npx figma-mcp                                    │
│    Found in: Claude Code                            │
│    Assign to mode: [Planning ▾]                     │
├─────────────────────────────────────────────────────┤
│ ⚠  linear          CONFLICT                         │
│    Different config in Claude Code vs Gemini CLI    │
│    [Review conflict →]                              │
└─────────────────────────────────────────────────────┘

[Cancel]                  [Import selected (4 of 5)]
```

Key design decisions:
- Mode assignment is inline per server, not a separate step
- Suggestions pre-populate mode assignments based on server name heuristics
- Conflicts don't block import of non-conflicting servers
- Deselection is possible — user might not want all of them

**Conflict state** — show diffs clearly:

```
Resolve conflict: linear

This server has different configurations in two tools.

GEMINI CLI                      CLAUDE CODE
────────────────────────────    ────────────────────────────
command: npx                    command: npx
args:    @linear/mcp-server     args:    @linear/mcp-server
env:     LINEAR_API_KEY=...     env:     (none)
                                         ^ missing env var

[Use Gemini CLI version]   [Use Claude Code version]   [Edit manually]
```

The diff view highlights the actual difference — don't make users figure it out.

---

### 9. Global Write Feedback

After any operation that writes to tool config files (mode switch, server add, manage, import complete) — show a consistent feedback strip:

```tsx
// components/agents/WriteFeedback.tsx

export function WriteFeedback({ result, onDismiss }: WriteFeedbackProps) {
  const hasRestartNeeded = result.tools.some(t => t.restartRequired && t.success)
  const hasErrors = result.tools.some(t => !t.success)

  return (
    <div className={`write-feedback ${hasErrors ? 'has-errors' : ''}`}>
      <div className="tool-results">
        {result.tools.map(tool => (
          <div key={tool.id} className="tool-result">
            {tool.success
              ? <CheckCircle size={14} className="success" />
              : <XCircle size={14} className="error" />
            }
            <span>{tool.name}</span>
            {tool.restartRequired && tool.success && (
              <span className="restart-badge">restart required</span>
            )}
            {!tool.success && (
              <span className="error-msg">{tool.error}</span>
            )}
          </div>
        ))}
      </div>

      {hasRestartNeeded && (
        <div className="restart-notice">
          ⚠ Restart {result.tools
            .filter(t => t.restartRequired && t.success)
            .map(t => t.name)
            .join(', ')
          } to apply changes
        </div>
      )}

      {!hasRestartNeeded && !hasErrors && (
        <button className="dismiss" onClick={onDismiss}>✕</button>
      )}
    </div>
  )
}
```

Auto-dismisses after 4 seconds if no restart warning and no errors. Restart warnings stay until dismissed manually.

---

## Page Layout and Visual Design

### Layout Structure

```
┌─────────────────────────────────────────────────────────────────┐
│  Agents                                    [Import] [+ Server]   │
│                                                                  │
│  ┌─ Active Mode ──────────────────────────────────────────────┐ │
│  │  ● Execution                               [Switch mode ▾] │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌─ Write feedback (transient) ───────────────────────────────┐ │
│  │  ✓ Claude Code  ✓ Gemini CLI  ⚠ restart required          │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  AI TOOLS ─────────────────────────────────────────────────────  │
│  [tool rows]                                                     │
│                                                                  │
│  MCP SERVERS ──────────────────────────────────────────────────  │
│  [server rows]                                                   │
│  [+ Add Server]                                                  │
│                                                                  │
│  MODES ─────────────────────────────────────────────────────── │
│  [mode rows]                                                     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Visual Design Principles

**Density without clutter.** This is a technical settings surface used by developers. Treat it like a terminal that got a good designer — information-dense, no wasted space, but with clear visual hierarchy. Think VS Code settings meets Linear — utilitarian but not ugly.

**Color carries meaning, not decoration.** Use color for three things only: status (green/yellow/red), active mode indicator, and mode color badges. Everything else is monochrome. This makes the status signals legible at a glance.

**Section headers as anchors.** With a long scrollable page, section headers (AI TOOLS, MCP SERVERS, MODES) should be sticky — they give orientation as you scroll. Use a subtle background blur for the sticky header so it doesn't visually collide with content below.

**Inline expansion over modals.** Forms expand inline, slide-overs for detail views. The only modal is conflict resolution because it genuinely requires a decision before proceeding. Everything else keeps spatial context.

**Code-friendly typography for technical values.** Config paths, commands, args, env var names — render these in `font-family: mono` with a subtle background pill. Makes scanning fast.

---

## State Management

All agent config state lives in a dedicated Zustand slice (or equivalent). Tauri commands are the data layer.

```tsx
// stores/agentConfigStore.ts

interface AgentConfigState {
  // Data
  servers: McpServerDef[]
  modes: Mode[]
  managedTools: ManagedTool[]
  detectedTools: DetectedTool[]
  activeMode: string | null

  // UI state
  scanning: boolean
  switching: boolean
  writeResults: WriteResult | null

  // Actions
  scan: () => Promise<void>
  manage: (toolId: string) => Promise<void>
  unmanage: (toolId: string) => Promise<void>
  addServer: (server: McpServerDef) => Promise<void>
  updateServer: (id: string, patch: Partial<McpServerDef>) => Promise<void>
  deleteServer: (id: string) => Promise<void>
  switchMode: (modeId: string) => Promise<void>
  importServers: (servers: McpServerDef[], modeAssignments: Record<string, string[]>) => Promise<void>
}
```

**Tauri commands required:**

```rust
#[tauri::command]
async fn mcp_scan_tools() -> Result<Vec<DetectedTool>, String>

#[tauri::command]
async fn mcp_manage_tool(tool_id: String) -> Result<WriteResult, String>

#[tauri::command]
async fn mcp_unmanage_tool(tool_id: String) -> Result<(), String>

#[tauri::command]
async fn mcp_switch_mode(mode_id: String) -> Result<ModeSwitchResult, String>

#[tauri::command]
async fn mcp_add_server(server: McpServerDef) -> Result<(), String>

#[tauri::command]
async fn mcp_update_server(id: String, patch: McpServerPatch) -> Result<(), String>

#[tauri::command]
async fn mcp_delete_server(id: String) -> Result<(), String>

#[tauri::command]
async fn mcp_import_scan() -> Result<ImportScanResult, String>

#[tauri::command]
async fn mcp_import_commit(servers: Vec<McpServerDef>, assignments: Vec<ModeAssignment>) -> Result<(), String>

#[tauri::command]
async fn mcp_get_tool_detail(tool_id: String) -> Result<ToolDetail, String>

#[tauri::command]
async fn mcp_verify_tool(tool_id: String) -> Result<ValidationReport, String>

#[tauri::command]
async fn mcp_restore_tool(tool_id: String) -> Result<(), String>
```

All commands are specta-typed — TypeScript types generated automatically.

---

## Error States

Every error needs three things: what happened, why it matters, what to do next.

```tsx
// components/agents/ErrorStates.tsx

export const errorMessages = {
  PARSE_ERROR: (tool: string, path: string, detail: string) => ({
    title: `${tool} config has invalid syntax`,
    detail: `${path} — ${detail}`,
    action: 'Fix the syntax error and try again. Shipwright has not modified this file.',
    severity: 'error' as const,
  }),

  PERMISSION_DENIED: (tool: string, path: string) => ({
    title: `Can't write to ${tool} config`,
    detail: `Permission denied: ${path}`,
    action: `Check file permissions: chmod 644 "${path}"`,
    severity: 'error' as const,
  }),

  VERIFICATION_FAILED: (tool: string, missing: string[]) => ({
    title: `${tool} config verification failed`,
    detail: `Written but could not verify: ${missing.join(', ')}`,
    action: 'Your backup has been restored. This may be a Shipwright bug — please report it.',
    severity: 'error' as const,
  }),

  TOOL_NOT_FOUND: (tool: string) => ({
    title: `${tool} not found`,
    detail: 'Binary not found in PATH',
    action: `Install ${tool} and restart Shipwright to detect it.`,
    severity: 'warning' as const,
  }),

  RESTART_REQUIRED: (tools: string[]) => ({
    title: 'Restart required',
    detail: `${tools.join(', ')} must be restarted to apply changes`,
    action: null,
    severity: 'info' as const,
  }),

  GEMINI_ENV_WARNING: () => ({
    title: 'Gemini env var note',
    detail: 'Gemini CLI does not inherit shell env vars automatically',
    action: 'Env vars must be declared in the server\'s env property. HTTP header values are not expanded.',
    severity: 'warning' as const,
  }),
}
```

---

## Implementation Order

Build in this exact sequence. Each step is independently usable.

**Phase 1 — Read-only foundation (2-3 days)**
1. `mcp_scan_tools` Tauri command — detect installed CLIs, find config paths
2. `AgentConfigStore` with scan state
3. Settings → Agents page scaffold — empty sections, loading states
4. AI Tools section — detected tools list, status dots, no actions yet
5. MCP Servers section — list from `.ship/config.toml`, read-only

**Phase 2 — Server management (2-3 days)**
6. Add Server form — inline, with ArgsInput + EnvVarsInput components
7. Edit Server — same form, pre-populated
8. Delete Server — with confirmation
9. `mcp_add_server`, `mcp_update_server`, `mcp_delete_server` commands
10. Server suggestions (built-in registry, inline suggestion card)

**Phase 3 — Tool management + write pipeline (2-3 days)**
11. Manage tool flow — click Manage → write pipeline → WriteFeedback
12. Tool Detail slide-over — raw config, per-server status, verify button
13. `mcp_manage_tool`, `mcp_verify_tool`, `mcp_restore_tool` commands
14. Mode switch from Agents page — calls existing mode switch, shows WriteFeedback

**Phase 4 — Import (2-3 days)**
15. `mcp_import_scan` command
16. Import Panel — scanning state + review state
17. Mode assignment inline per server
18. Conflict detection + ConflictState UI
19. `mcp_import_commit` command

**Phase 5 — Polish (1-2 days)**
20. Onboarding card (first open, no config)
21. Error states — all error messages, error UI component
22. Gemini env var warning
23. Codex `mcp_servers` typo detection
24. Empty states throughout
25. Sticky section headers
26. Keyboard navigation (Tab through form fields, Esc closes panels)

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 0.1 | 2026-02-22 | Alpha scope — Claude Code, Gemini CLI, Codex. Full component breakdown, import UX, implementation order. |
