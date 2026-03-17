# Provider Config Surface — Gap Analysis

> Research date: 2026-03-17
> Purpose: Map everything each provider supports, diff against what Ship compiles, identify gaps.
> Sources: Upstream JSON Schemas, official docs, SchemaStore, GitHub repos.

---

## How to Read This Document

Each provider section lists the **full upstream config surface**, then marks each field:
- **✅ Compiled** — Ship emits this today
- **🗺️ Maps to Ship concept** — has a Ship equivalent but not yet compiled
- **🔧 Needs provider_config** — provider-specific, needs passthrough
- **⬜ Out of scope** — UX/cosmetic, not relevant to workspace config
- **🔴 Gap** — meaningful capability Ship should support but doesn't

---

## Summary: Ship Coverage by Provider

| Provider | Total Config Surfaces | Ship Compiles | Coverage | Critical Gaps |
|---|---|---|---|---|
| Claude Code | ~80 fields + 18 hook triggers | ~20 fields + 6 triggers | ~25% | model picker, plugins, env, managed settings, 12 hook triggers |
| Gemini CLI | ~120 fields + 11 hook triggers | ~15 fields + 5 triggers | ~12% | model, approval mode, sandbox, browser agent, extensions, env |
| Codex CLI | ~90 fields + 50 feature flags | ~5 fields (MCP + rules only) | ~6% | model, approval, sandbox, permissions profiles, multi-agent, plugins |
| Cursor | ~30 fields (file-based) + SQLite | ~15 fields | ~50% | model (SQLite), YOLO mode, environment.json, .cursorignore |

---

## 1. Claude Code

### Upstream Schema
`https://json.schemastore.org/claude-code-settings.json`

### File Tree (full)

```
~/.claude/
├── CLAUDE.md                           # global instructions
├── settings.json                       # user settings
├── settings.local.json                 # user local (gitignored)
├── skills/*/SKILL.md                   # global skills
├── keybindings.json                    # key bindings
├── plans/                              # plan storage
├── projects/<hash>/
│   ├── CLAUDE.md                       # per-project instructions override
│   └── settings.json                   # per-project settings override
└── todos/                              # managed state

~/.claude.json                          # user MCP servers

/etc/claude-code/managed-settings.json  # enterprise managed settings

<project>/
├── CLAUDE.md                           # project instructions (committed)
├── CLAUDE.local.md                     # personal project instructions (gitignored)
├── <subdir>/CLAUDE.md                  # directory-scoped instructions
├── .claude/
│   ├── settings.json                   # project settings (committed)
│   ├── settings.local.json             # project local (gitignored)
│   ├── rules/*.md                      # path-scoped rules (glob matching)
│   ├── skills/*/SKILL.md              # project skills
│   ├── agents/*.md                     # sub-agents (team definitions)
│   └── commands/*.md                   # slash commands (legacy)
└── .mcp.json                           # MCP servers
```

### settings.json — Full Field Map

#### Core Agent Config
| Field | Type | Ship Status | Notes |
|---|---|---|---|
| `model` | string | ✅ Compiled | Via settings patch |
| `availableModels` | string[] | 🔧 provider_config | Restrict model picker |
| `effortLevel` | "low"/"medium"/"high" | 🔧 provider_config | Reasoning effort |
| `fastMode` | boolean | 🔧 provider_config | Fast output mode |
| `fastModePerSessionOptIn` | boolean | 🔧 provider_config | |
| `alwaysThinkingEnabled` | boolean | 🔧 provider_config | |
| `language` | string | 🔧 provider_config | Response language |
| `outputStyle` | string | 🔧 provider_config | Response style |
| `teammateMode` | "auto"/"in-process"/"tmux" | 🔧 provider_config | Multi-agent topology |

#### Permissions
| Field | Type | Ship Status | Notes |
|---|---|---|---|
| `permissions.allow` | string[] | ✅ Compiled | Tool allowlist |
| `permissions.deny` | string[] | ✅ Compiled | Tool denylist |
| `permissions.ask` | string[] | ✅ Compiled | Ask-before-use |
| `permissions.defaultMode` | enum | ✅ Compiled | Agent approval mode |
| `permissions.additionalDirectories` | string[] | ✅ Compiled | Extra directories |
| `permissions.disableBypassPermissionsMode` | "disable" | 🔴 Gap | Enterprise lockdown |

#### Hooks
| Trigger | Ship Status | Notes |
|---|---|---|
| `PreToolUse` | ✅ Compiled | |
| `PostToolUse` | ✅ Compiled | |
| `Notification` | ✅ Compiled | |
| `Stop` | ✅ Compiled | |
| `SubagentStop` | ✅ Compiled | |
| `PreCompact` | ✅ Compiled | |
| `SessionStart` | 🔴 Gap | Workspace setup automation |
| `SessionEnd` | 🔴 Gap | Cleanup |
| `PostToolUseFailure` | 🔴 Gap | Error handling |
| `PermissionRequest` | 🔴 Gap | Permission audit |
| `UserPromptSubmit` | 🔴 Gap | Input processing |
| `SubagentStart` | 🔴 Gap | Agent lifecycle |
| `Setup` | 🔴 Gap | Init/maintenance hooks |
| `InstructionsLoaded` | 🔴 Gap | Audit/logging |
| `ConfigChange` | 🔴 Gap | Hot-reload triggers |
| `WorktreeCreate` | 🔴 Gap | Worktree management |
| `WorktreeRemove` | 🔴 Gap | Worktree cleanup |
| `TeammateIdle` | 🔴 Gap | Multi-agent coordination |
| `TaskCompleted` | 🔴 Gap | Task lifecycle gates |

**Hook command types (4):**
| Type | Ship Status | Notes |
|---|---|---|
| `command` | ✅ Compiled | Shell command execution |
| `prompt` | 🔴 Gap | LLM prompt evaluation |
| `agent` | 🔴 Gap | Multi-turn agent hook |
| `http` | 🔴 Gap | Webhook/HTTP callback |

#### MCP Server Controls
| Field | Type | Ship Status | Notes |
|---|---|---|---|
| `enableAllProjectMcpServers` | boolean | 🔧 provider_config | Auto-trust project MCP |
| `enabledMcpjsonServers` | string[] | 🔧 provider_config | Explicit enable list |
| `disabledMcpjsonServers` | string[] | 🔧 provider_config | Explicit disable list |
| `allowedMcpServers` | object[] | 🔧 provider_config | Enterprise allowlist |
| `deniedMcpServers` | object[] | 🔧 provider_config | Enterprise denylist |
| `allowManagedMcpServersOnly` | boolean | ⬜ Managed only | |

#### Environment & Auth
| Field | Type | Ship Status | Notes |
|---|---|---|---|
| `env` | Record<string, string> | 🔴 Gap | Environment variables |
| `apiKeyHelper` | string | 🔧 provider_config | Auth script |
| `awsCredentialExport` | string | 🔧 provider_config | AWS creds |
| `awsAuthRefresh` | string | 🔧 provider_config | AWS refresh |
| `forceLoginMethod` | "claudeai"/"console" | 🔧 provider_config | |
| `forceLoginOrgUUID` | string | 🔧 provider_config | |
| `otelHeadersHelper` | string | 🔧 provider_config | OTEL auth |

#### Plugins/Marketplace
| Field | Type | Ship Status | Notes |
|---|---|---|---|
| `enabledPlugins` | object | 🗺️ Maps to plugins manifest | Plugin install state |
| `pluginConfigs` | object | 🔧 provider_config | Per-plugin config |
| `extraKnownMarketplaces` | object | 🔧 provider_config | |
| `strictKnownMarketplaces` | array | ⬜ Managed only | |
| `skippedMarketplaces` | array | 🔧 provider_config | |
| `skippedPlugins` | array | 🔧 provider_config | |
| `blockedMarketplaces` | array | ⬜ Managed only | |

#### Agent Limits
| Field | Type | Ship Status | Notes |
|---|---|---|---|
| `maxCostPerSession` | number | ✅ Compiled | Cost cap |
| `maxTurns` | number | ✅ Compiled | Turn limit |

#### UX/UI (mostly out of scope)
| Field | Ship Status | Notes |
|---|---|---|
| `statusLine` | ⬜ | Terminal status display |
| `fileSuggestion` | ⬜ | File suggestion command |
| `spinnerVerbs` | ⬜ | Loading text |
| `spinnerTipsEnabled` | ⬜ | |
| `terminalProgressBarEnabled` | ⬜ | |
| `showTurnDuration` | ⬜ | |
| `prefersReducedMotion` | ⬜ | |
| `autoUpdatesChannel` | ⬜ | |
| `cleanupPeriodDays` | ⬜ | |
| `respectGitignore` | ⬜ | |
| `autoMemoryEnabled` | ⬜ | |
| `plansDirectory` | ⬜ | |
| `skipWebFetchPreflight` | ⬜ | |
| `sandbox` | 🔧 provider_config | Sandbox config |

#### Attribution
| Field | Type | Ship Status | Notes |
|---|---|---|---|
| `attribution.commit` | string | 🔧 provider_config | Commit message suffix |
| `attribution.pr` | string | 🔧 provider_config | PR description suffix |
| `includeGitInstructions` | boolean | 🔧 provider_config | |
| `includeCoAuthoredBy` | boolean | 🔧 provider_config | Deprecated |

### Sub-agents (.claude/agents/*.md)
| Feature | Ship Status | Notes |
|---|---|---|
| Agent definition files | ✅ Compiled (Claude only) | `.claude/agents/<name>.md` |
| Cross-provider agent format | 🔴 Gap | Only compiles for Claude |

### Commands (.claude/commands/*.md)
| Feature | Ship Status | Notes |
|---|---|---|
| Slash commands | ⬜ Out of scope | Legacy, superseded by skills |

---

## 2. Gemini CLI

### Upstream Schema
`https://raw.githubusercontent.com/google-gemini/gemini-cli/main/schemas/settings.schema.json`

### File Tree (full)

```
/etc/gemini-cli/system-defaults.json    # system defaults (lowest precedence)
/etc/gemini-cli/settings.json           # system overrides (highest precedence)

~/.gemini/
├── settings.json                       # user settings
├── GEMINI.md                           # global instructions
├── policies/*.toml                     # global permission policies
├── extensions/<name>/
│   ├── gemini-extension.json           # extension manifest
│   ├── GEMINI.md                       # extension context
│   ├── commands/*.toml                 # extension slash commands
│   └── hooks/hooks.json               # extension hooks
└── .env                                # global env vars

<project>/
├── GEMINI.md                           # project instructions
├── <subdir>/GEMINI.md                  # directory-scoped instructions
├── .gemini/
│   ├── settings.json                   # project settings
│   ├── .env                            # project env vars
│   ├── policies/*.toml                 # project policies (higher precedence)
│   ├── sandbox.Dockerfile              # custom Docker sandbox
│   └── sandbox-macos-*.sb             # macOS sandbox profiles
├── .geminiignore                       # AI context filtering
└── .env                                # environment variables (walk up to git root)
```

### settings.json — Full Field Map

#### Core Agent Config
| Field | Type | Ship Status | Notes |
|---|---|---|---|
| `model.name` | string | 🔴 Gap | Model selection |
| `model.maxSessionTurns` | number | 🔴 Gap | Turn limit |
| `model.compressionThreshold` | number (0-1) | 🔧 provider_config | Context compaction |
| `model.disableLoopDetection` | boolean | 🔧 provider_config | |
| `model.summarizeToolOutput` | object | 🔧 provider_config | Output summarization |
| `general.defaultApprovalMode` | "default"/"auto_edit"/"plan" | 🔴 Gap | Agent approval mode |
| `general.maxAttempts` | number | 🔧 provider_config | Retry limit |
| `general.checkpointing.enabled` | boolean | 🔧 provider_config | |
| `general.sessionRetention.*` | object | 🔧 provider_config | Session cleanup |

#### Hooks
| Trigger | Ship Equivalent | Ship Status | Notes |
|---|---|---|---|
| `BeforeTool` | `PreToolUse` | ✅ Compiled | |
| `AfterTool` | `PostToolUse` | ✅ Compiled | |
| `Notification` | `Notification` | ✅ Compiled | |
| `SessionEnd` | `Stop` | ✅ Compiled | |
| `PreCompress` | `PreCompact` | ✅ Compiled | |
| `SessionStart` | — | 🔴 Gap | |
| `BeforeAgent` | — | 🔴 Gap | Agent lifecycle |
| `AfterAgent` | — | 🔴 Gap | |
| `BeforeModel` | — | 🔴 Gap | Model invocation |
| `AfterModel` | — | 🔴 Gap | |
| `BeforeToolSelection` | — | 🔴 Gap | Tool routing |

#### MCP Servers
| Feature | Ship Status | Notes |
|---|---|---|
| `mcpServers` (stdio) | ✅ Compiled | command/args/env |
| `mcpServers` (SSE) | ✅ Compiled | `url` field |
| `mcpServers` (HTTP) | ✅ Compiled | `httpUrl` field |
| MCP timeout config | 🔴 Gap | Server-level timeouts |

#### Permissions / Policies
| Feature | Ship Status | Notes |
|---|---|---|
| `[[tool_policies]]` TOML | ✅ Compiled | To `.gemini/policies/ship.toml` |
| `policyPaths` | 🔧 provider_config | Extra policy directories |
| `adminPolicyPaths` | ⬜ Managed only | |

#### Context / Discovery
| Field | Ship Status | Notes |
|---|---|---|
| `context.fileName` | ⬜ | Custom context filename |
| `context.discoveryMaxDirs` | ⬜ | Directory scan limit |
| `context.includeDirectories` | 🔧 provider_config | Multi-repo context |
| `context.fileFiltering.*` | ⬜ | Gitignore/geminiignore behavior |

#### Browser Agent
| Field | Ship Status | Notes |
|---|---|---|
| `agents.browser.sessionMode` | 🔧 provider_config | "persistent"/"isolated"/"existing" |
| `agents.browser.headless` | 🔧 provider_config | |
| `agents.browser.allowedDomains` | 🔧 provider_config | |
| `agents.browser.visualModel` | 🔧 provider_config | |

#### Extensions
| Feature | Ship Status | Notes |
|---|---|---|
| Extension manifests | 🔴 Gap | `gemini-extension.json` |
| Extension context | 🔴 Gap | Per-extension GEMINI.md |
| Extension slash commands | 🔴 Gap | `.toml` command definitions |
| Extension hooks | 🔴 Gap | `hooks/hooks.json` |

#### Environment
| Feature | Ship Status | Notes |
|---|---|---|
| `.gemini/.env` | 🔴 Gap | Project env vars |
| `~/.gemini/.env` | 🔴 Gap | Global env vars |
| `.env` walk-up | 🔴 Gap | Directory-scoped env |

#### Sandbox
| Field | Ship Status | Notes |
|---|---|---|
| `tools.sandbox` | 🔧 provider_config | Sandbox mode |
| `.gemini/sandbox.Dockerfile` | 🔧 provider_config | Custom sandbox image |
| `security.hideSensitiveEnvVars` | ⬜ | |

#### UI/Telemetry (out of scope)
Fields: `ui.*` (theme, footer, spinner, accessibility), `privacy.*`, `telemetry.*`, `billing.*`, `ide.*`, `output.format`

---

## 3. OpenAI Codex CLI

### Upstream Schema
`https://raw.githubusercontent.com/openai/codex/main/codex-rs/core/config.schema.json`

### File Tree (full)

```
~/.codex/
├── config.toml                         # user settings
├── AGENTS.md                           # global instructions
└── instructions.md                     # alt instructions location

<project>/
├── AGENTS.md                           # project instructions (walks git root → CWD)
├── codex.md                            # alt instructions
├── .codex/
│   └── config.toml                     # project config
├── .agents/
│   └── skills/*/SKILL.md              # skills
└── agents/
    └── *.toml                          # multi-agent role configs
```

### config.toml — Full Field Map

#### Core Agent Config
| Field | Type | Ship Status | Notes |
|---|---|---|---|
| `model` | string | 🔴 Gap | Model selection |
| `model_provider` | string | 🔴 Gap | Provider (openai, azure, etc.) |
| `model_context_window` | int64 | 🔧 provider_config | |
| `model_reasoning_effort` | enum | 🔧 provider_config | none/minimal/low/medium/high/xhigh |
| `model_reasoning_summary` | enum | 🔧 provider_config | auto/concise/detailed/none |
| `model_verbosity` | enum | 🔧 provider_config | low/medium/high |
| `model_auto_compact_token_limit` | int64 | 🔧 provider_config | |
| `approval_policy` | enum | 🔴 Gap | suggest/auto-edit/full-auto |
| `sandbox_mode` | enum | 🔴 Gap | read-only/workspace-write/danger-full-access |
| `profile` | string | 🔧 provider_config | Active config profile |
| `profiles` | object | 🔧 provider_config | Named config profiles |
| `personality` | enum | 🔧 provider_config | none/friendly/pragmatic |
| `instructions` | string | 🔧 provider_config | System instructions |
| `developer_instructions` | string | 🔧 provider_config | Developer role message |
| `compact_prompt` | string | 🔧 provider_config | Compaction prompt |
| `tool_output_token_limit` | uint | 🔧 provider_config | |

#### MCP Servers
| Feature | Ship Status | Notes |
|---|---|---|
| `[mcp_servers.<name>]` (stdio) | ✅ Compiled | command/args/env |
| `[mcp_servers.<name>]` (HTTP) | ✅ Compiled | url field |
| `startup_timeout_sec` | 🔴 Gap | Per-server timeout |
| `tool_timeout_sec` | 🔴 Gap | Per-tool timeout |
| `enabled` | 🔴 Gap | Server enable/disable |
| `mcp_oauth_callback_port` | 🔧 provider_config | |
| `mcp_oauth_callback_url` | 🔧 provider_config | |
| `mcp_oauth_credentials_store` | 🔧 provider_config | |

#### Permissions
| Field | Ship Status | Notes |
|---|---|---|
| `default_permissions` | 🔴 Gap | Named permissions profile |
| `permissions` | 🔴 Gap | `PermissionsToml` object |
| Feature flags (50+ booleans) | 🔧 provider_config | `features.*` toggles |

#### Multi-Agent
| Feature | Ship Status | Notes |
|---|---|---|
| `agents` config | 🔴 Gap | Agent definitions in TOML |
| `agents/*.toml` files | 🔴 Gap | Role-specific configs |
| `approvals_reviewer` | 🔧 provider_config | user/guardian_subagent |

#### Skills
| Feature | Ship Status | Notes |
|---|---|---|
| `.agents/skills/*/SKILL.md` | ✅ Compiled | Standard skill format |
| `skills` config object | 🔧 provider_config | Skill toggle/config |

#### Plugins
| Feature | Ship Status | Notes |
|---|---|---|
| `plugins` | 🔴 Gap | Plugin configurations |

#### Environment
| Field | Ship Status | Notes |
|---|---|---|
| `shell_environment_policy` | 🔧 provider_config | inherit/clean |
| `env` (via shell) | 🔴 Gap | No env passthrough |
| `openai_base_url` | 🔧 provider_config | |
| `chatgpt_base_url` | 🔧 provider_config | |

#### Observability
| Field | Ship Status | Notes |
|---|---|---|
| `otel.otlp_endpoint` | 🔧 provider_config | |
| `otel.*` | 🔧 provider_config | Full OTEL config |
| `notify` | 🔧 provider_config | External notification command |
| `log_dir` | 🔧 provider_config | |

#### Experimental / Advanced
| Field | Ship Status | Notes |
|---|---|---|
| `audio.*` | ⬜ | Realtime audio |
| `realtime.*` | ⬜ | WebSocket mode |
| `web_search` | 🔧 provider_config | disabled/cached/live |
| `history.*` | 🔧 provider_config | Persistence settings |
| `memories.*` | 🔧 provider_config | Memory subsystem |
| `ghost_snapshot.*` | ⬜ | Snapshot feature |
| `tui.*` | ⬜ | Terminal UI |
| `apps.*` | 🔧 provider_config | App-specific controls |
| `tools.*` | 🔧 provider_config | Tool feature toggles |

---

## 4. Cursor

### No Upstream Schema
Cursor does not publish JSON schemas for any config file.

### File Tree (full)

```
~/.cursor/
├── mcp.json                            # global MCP servers
├── hooks.json                          # global hooks
├── cli-config.json                     # global CLI permissions (DANGEROUS)
├── skills/*/SKILL.md                   # global skills
└── state.vscdb                         # SQLite (all AI settings, models, API keys)

<project>/
├── .cursor/
│   ├── mcp.json                        # project MCP servers
│   ├── rules/*.mdc                     # rules (MDC format)
│   ├── hooks.json                      # project hooks
│   ├── cli.json                        # project CLI permissions
│   ├── skills/*/SKILL.md              # project skills
│   └── environment.json                # cloud agent environment
├── .cursorrules                        # legacy rules (deprecated)
├── .cursorignore                       # AI + indexing exclusion
├── .cursorindexingignore               # indexing-only exclusion
└── .vscode/settings.json               # workspace settings (some Cursor keys)
```

### Config Surface — Full Field Map

#### MCP Servers (`.cursor/mcp.json`)
| Feature | Ship Status | Notes |
|---|---|---|
| `mcpServers` (stdio) | ✅ Compiled | command/args/env |
| `mcpServers` (remote) | ✅ Compiled | url/headers |
| Global MCP (`~/.cursor/mcp.json`) | ⬜ | User responsibility |

#### Rules (`.cursor/rules/*.mdc`)
| Feature | Ship Status | Notes |
|---|---|---|
| `alwaysApply` | ✅ Compiled | Default true for Ship rules |
| `description` | ✅ Compiled | From rule metadata |
| `globs` | ✅ Compiled | File pattern matching |
| Rule body (markdown) | ✅ Compiled | Rule content |

#### Hooks (`.cursor/hooks.json`)
| Trigger | Ship Equivalent | Ship Status | Notes |
|---|---|---|---|
| `beforeShellExecution` | `PreToolUse` | ✅ Compiled | |
| `beforeMCPExecution` | `PreToolUse` | ✅ Compiled | Split from same Ship trigger |
| `afterShellExecution` | `PostToolUse` | ✅ Compiled | |
| `afterMCPExecution` | `PostToolUse` | ✅ Compiled | |
| `afterFileEdit` | — | 🔴 Gap | Post-edit formatting/staging |
| `beforeReadFile` | — | 🔴 Gap | Content rewriting |
| `stop` | `Stop` | ✅ Compiled | |

#### Permissions (`.cursor/cli.json`)
| Feature | Ship Status | Notes |
|---|---|---|
| `permissions.allow` | ✅ Compiled | Shell/Read/Write/WebFetch/Mcp |
| `permissions.deny` | ✅ Compiled | Same typed patterns |
| Global permissions (`~/.cursor/cli-config.json`) | ⬜ | User responsibility |

#### Models & AI Settings
| Feature | Ship Status | Notes |
|---|---|---|
| Model selection | 🔴 Gap (SQLite) | Not file-configurable |
| API keys (OpenAI, Anthropic, Google, Azure, AWS) | 🔴 Gap (SQLite) | Not file-configurable |
| Custom model endpoint | 🔴 Gap (SQLite) | OpenAI-compatible base URL |
| YOLO mode | 🔴 Gap (SQLite) | Auto-approve all |
| MAX mode / Auto mode | ⬜ | Per-request model selection |

#### Context Features
| Feature | Ship Status | Notes |
|---|---|---|
| `.cursorignore` | 🔴 Gap | AI context exclusion |
| `.cursorindexingignore` | ⬜ | Indexing performance |
| `@-mentions` (files, rules, notepads) | ⬜ | Interactive-only |

#### Cloud Agents
| Feature | Ship Status | Notes |
|---|---|---|
| `environment.json` | 🔴 Gap | Cloud agent setup recipe |
| Background agents | ⬜ | Cloud compute |
| Shadow workspace | ⬜ | Background verification |

---

## Cross-Provider Gap Summary

### Critical Gaps (affect user migration from provider → Ship)

| Gap | Providers Affected | Ship Concept Mapping | Priority |
|---|---|---|---|
| **`provider_config` passthrough** | ALL | Free-form per-provider settings blob | **P0** |
| **Environment variables** | Claude (`env`), Gemini (`.env`), Codex (shell policy) | Ship `[env]` section in preset? | **P0** |
| **Model selection** | Gemini, Codex, Cursor | Already in preset `[profile]` but only compiled for Claude | **P1** |
| **Approval mode** | Gemini (`defaultApprovalMode`), Codex (`approval_policy`) | Maps to Ship workspace modes | **P1** |
| **Sandbox mode** | Codex (`sandbox_mode`), Gemini (`tools.sandbox`) | Ship concept TBD | **P2** |
| **MCP timeouts** | Codex (`startup_timeout_sec`, `tool_timeout_sec`) | Ship MCP server config | **P2** |
| **MCP enable/disable** | Codex (`enabled`), Claude (`enabledMcpjsonServers`) | Ship MCP server config | **P2** |
| **Hook triggers (12 new)** | Claude (12 new), Gemini (6 new), Cursor (2 new) | Expand Ship hook trigger enum | **P2** |
| **Hook types (3 new)** | Claude (`prompt`, `agent`, `http`) | Ship hook type enum | **P2** |
| **Multi-agent roles** | Codex (`agents`, `*.toml`), Claude (`.claude/agents/`) | Ship team/agent concept | **P2** |
| **Plugins** | Claude (`enabledPlugins`), Codex (`plugins`) | Ship plugins manifest (partial) | **P2** |
| **Extensions** | Gemini (extension system) | No Ship equivalent | **P3** |
| **.cursorignore** | Cursor | Ship ignore concept TBD | **P3** |

### What Ship Does Well (no gaps)

| Feature | Ship Approach | Notes |
|---|---|---|
| MCP servers (basic) | Unified TOML → per-provider format | All 4 providers |
| Rules/context files | `.ship/agents/rules/` → concatenated or per-file | All 4 providers |
| Skills | `.ship/agents/skills/` → provider skill dirs | All 4 providers |
| Permissions (Claude) | Lossless translation | Best coverage |
| Permissions (Gemini) | Policy engine TOML | Good coverage |
| Permissions (Cursor) | Typed CLI patterns | Good coverage |
| Hooks (basic triggers) | 6 triggers compiled | Claude, Gemini, Cursor |
| Agent limits | `maxCostPerSession`, `maxTurns` | Claude only |
| Multi-provider compilation | Single source → 4 outputs | Core value prop |

### Honest Assessment

**What we tell users today:** "Ship replaces your provider config files. Define once in `.ship/`, compile to all providers."

**What's actually true:** Ship handles the *structural* config well (MCP, rules, skills, permissions, basic hooks) but misses most *behavioral* config (model selection, approval modes, sandbox, env vars, timeouts, plugins). A user migrating a mature Claude Code setup would lose ~75% of their settings.json fields. A Codex user would lose approval_policy and sandbox_mode — arguably their most important settings.

**The path forward:**
1. **P0: `provider_config` passthrough** — Let users put arbitrary provider-specific settings in their preset. Ship passes them through unmodified. This closes the "long tail" gap immediately.
2. **P1: First-class model + approval mode** — These are universal concepts. Every provider has them. Compile `[profile] model` to all providers, not just Claude. Map Ship workspace modes to provider approval modes.
3. **P2: Incremental coverage** — Hook triggers, MCP timeouts, env vars, multi-agent. Add as demand appears.
4. **P3: Provider-specific features** — Extensions (Gemini), cloud agents (Cursor). Only if users ask.

---

## Appendix: Schema URLs

| Provider | Schema | URL |
|---|---|---|
| Claude Code | settings.json | `https://json.schemastore.org/claude-code-settings.json` |
| Gemini CLI | settings.json | `https://raw.githubusercontent.com/google-gemini/gemini-cli/main/schemas/settings.schema.json` |
| Codex CLI | config.toml | `https://raw.githubusercontent.com/openai/codex/main/codex-rs/core/config.schema.json` |
| Cursor | — | No published schemas |
