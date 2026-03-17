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
| Codex CLI | ~90 fields + 50 feature flags + Starlark rules | ~5 fields (MCP + context only) | ~6% | model, approval/sandbox, Starlark rules, permission profiles (fs+net ACLs), granular approval, multi-agent, per-MCP auth/tools |
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
| `BeforeAgent` | — | 🔴 Gap | Sub-agent lifecycle |
| `AfterAgent` | — | 🔴 Gap | Per-turn post-model |
| `BeforeModel` | — | 🔴 Gap | Pre-LLM-request |
| `AfterModel` | — | 🔴 Gap | Per-chunk post-LLM |
| `BeforeToolSelection` | — | 🔴 Gap | Tool routing gate |

Hook fields: `matcher` (tool name pattern), `command` (script), `timeout` (default 60s).
Communication: JSON on stdin/stdout. Env vars: `GEMINI_PROJECT_DIR`, `GEMINI_SESSION_ID`, `GEMINI_CWD`.

#### MCP Servers
| Feature | Ship Status | Notes |
|---|---|---|
| `mcpServers` (stdio) | ✅ Compiled | command/args/env |
| `mcpServers` (SSE) | ✅ Compiled | `url` field |
| `mcpServers` (HTTP) | ✅ Compiled | `httpUrl` field |
| `timeout` per server | 🔴 Gap | Connection timeout (ms) |
| `cwd` per server | 🔴 Gap | Working directory |
| `trust` per server | 🔧 provider_config | Trust level |
| `includeTools` per server | 🔴 Gap | Tool allowlist per MCP server |
| `excludeTools` per server | 🔴 Gap | Tool blocklist per MCP server |
| `headers` (HTTP/SSE) | 🔴 Gap | Auth headers |
| Global `mcp.allowed` | 🔧 provider_config | Server-level allowlist |
| Global `mcp.excluded` | 🔧 provider_config | Server-level blocklist |

#### Permissions / Policies
| Feature | Ship Status | Notes |
|---|---|---|
| `[[tool_policies]]` TOML | ✅ Compiled | To `.gemini/policies/ship.toml` (tool, pattern, decision) |
| `mcpName` field in policies | 🔴 Gap | Target specific MCP server in policy rules |
| `argsPattern` field in policies | 🔴 Gap | Regex against JSON-serialized tool args |
| `priority` field in policies | 🔴 Gap | 0-999 priority within tier (3-tier system) |
| `commandPrefix` field in policies | 🔴 Gap | Shell command prefix matching |
| `policyPaths` | 🔧 provider_config | Extra policy directories |
| `adminPolicyPaths` | ⬜ Managed only | |

#### Tool Controls
| Field | Ship Status | Notes |
|---|---|---|
| `tools.core` | 🔧 provider_config | Built-in tool allowlist |
| `tools.allowed` | 🗺️ Maps to permissions | Tools bypassing confirmation e.g. `"run_shell_command(git)"` |
| `tools.exclude` | 🗺️ Maps to permissions | Tools to disable |
| `tools.discoveryCommand` | 🔧 provider_config | Custom tool discovery |
| `tools.callCommand` | 🔧 provider_config | Custom tool call command (JSON stdin/stdout) |
| `tools.truncateToolOutputThreshold` | 🔧 provider_config | Default 40000 |
| `tools.shell.*` | 🔧 provider_config | Shell behavior (interactive, pager, timeout) |

#### Context / Discovery
| Field | Ship Status | Notes |
|---|---|---|
| `context.fileName` | ⬜ | Custom context filename (default "GEMINI.md") |
| `context.discoveryMaxDirs` | ⬜ | Directory scan limit (default 200) |
| `context.includeDirectories` | 🔧 provider_config | Multi-repo context |
| `context.loadMemoryFromIncludeDirectories` | 🔧 provider_config | |
| `context.fileFiltering.*` | ⬜ | Gitignore/geminiignore behavior |
| `@path/to/file.md` imports in GEMINI.md | ⬜ | Context file imports |

#### Security / Enterprise
| Field | Ship Status | Notes |
|---|---|---|
| `security.disableYoloMode` | 🔧 provider_config | Enterprise lockdown |
| `security.disableAlwaysAllow` | 🔧 provider_config | |
| `security.enablePermanentToolApproval` | 🔧 provider_config | |
| `security.folderTrust.enabled` | 🔧 provider_config | |
| `security.environmentVariableRedaction.*` | 🔧 provider_config | Sensitive var masking |
| `security.blockGitExtensions` | 🔧 provider_config | |
| `security.allowedExtensions` | 🔧 provider_config | Regex patterns |

#### Browser Agent
| Field | Ship Status | Notes |
|---|---|---|
| `agents.browser.sessionMode` | 🔧 provider_config | "persistent"/"isolated"/"existing" |
| `agents.browser.headless` | 🔧 provider_config | |
| `agents.browser.allowedDomains` | 🔧 provider_config | Default: github, google, localhost |
| `agents.browser.visualModel` | 🔧 provider_config | |

#### Extensions
| Feature | Ship Status | Notes |
|---|---|---|
| Extension manifests | 🔴 Gap | `gemini-extension.json` |
| Extension context | 🔴 Gap | Per-extension GEMINI.md |
| Extension slash commands | 🔴 Gap | `.toml` command definitions |
| Extension hooks | 🔴 Gap | `hooks/hooks.json` |
| Extension policies | 🔴 Gap | Per-extension `policies/*.toml` |

#### Custom Slash Commands
| Feature | Ship Status | Notes |
|---|---|---|
| `.gemini/commands/*.toml` | 🗺️ Maps to skills | Project commands |
| `~/.gemini/commands/*.toml` | 🗺️ Maps to skills | User commands |
| Template: `@{path}`, `!{cmd}`, `{{args}}` | 🔴 Gap | Template syntax not in Ship skills |

#### Environment
| Feature | Ship Status | Notes |
|---|---|---|
| `.gemini/.env` | 🔴 Gap | Project env vars |
| `~/.gemini/.env` | 🔴 Gap | Global env vars |
| `.env` walk-up | 🔴 Gap | Directory-scoped env |

#### Sandbox
| Field | Ship Status | Notes |
|---|---|---|
| `tools.sandbox` | 🔧 provider_config | true/"docker"/"podman"/path/object |
| `.gemini/sandbox.Dockerfile` | 🔧 provider_config | Custom sandbox image |
| `.gemini/sandbox-macos-*.sb` | 🔧 provider_config | macOS sandbox profiles |

#### UI/Telemetry (out of scope)
Fields: `ui.*` (theme, footer, spinner, accessibility ~30 fields), `privacy.*`, `telemetry.*`, `billing.*`, `ide.*`, `output.format`, `model.summarizeToolOutput`, `model.compressionThreshold`, `model.disableLoopDetection`

---

## 3. OpenAI Codex CLI

### Upstream Schema
`https://raw.githubusercontent.com/openai/codex/main/codex-rs/core/config.schema.json`

### File Tree (full)

```
/etc/codex/managed_config.toml          # admin/enterprise policy (highest precedence)

~/.codex/
├── config.toml                         # user settings
├── AGENTS.md                           # global instructions
├── AGENTS.override.md                  # global override (takes precedence)
├── rules/*.rules                       # execution policy rules (Starlark)
├── skills/*/SKILL.md                   # user skills
└── auth.json / keyring                 # credentials

<project>/
├── AGENTS.md                           # project instructions (walks git root → CWD)
├── AGENTS.override.md                  # override at each directory level
├── .codex/
│   └── config.toml                     # project config (trusted projects only)
├── .agents/
│   └── skills/*/SKILL.md              # repo skills
└── agents/
    └── *.toml                          # multi-agent role configs
```

**Resolution order** (highest wins): CLI `-c` flags > project configs (closest-to-cwd) > user config > managed config > defaults.

### config.toml — Full Field Map

#### Core Agent Config
| Field | Type | Ship Status | Notes |
|---|---|---|---|
| `model` | string | 🔴 Gap | Model selection |
| `model_provider` | string | 🔴 Gap | Provider (openai, azure, etc.) |
| `model_providers` | map | 🔧 provider_config | Custom provider definitions (name, base_url, env_key, wire_api, headers) |
| `model_context_window` | int64 | 🔧 provider_config | |
| `model_reasoning_effort` | enum | 🔧 provider_config | none/minimal/low/medium/high/xhigh |
| `model_reasoning_summary` | enum | 🔧 provider_config | auto/concise/detailed/none |
| `model_verbosity` | enum | 🔧 provider_config | low/medium/high (GPT-5) |
| `model_auto_compact_token_limit` | int64 | 🔧 provider_config | |
| `approval_policy` | union | 🔴 Gap | "untrusted"/"on-request"/"never" or `{granular: ...}` |
| `sandbox_mode` | enum | 🔴 Gap | read-only/workspace-write/danger-full-access |
| `sandbox_workspace_write` | object | 🔴 Gap | writable_roots, network_access |
| `profile` | string | 🔧 provider_config | Active config profile |
| `profiles` | map | 🔧 provider_config | Named profiles (override almost any field) |
| `personality` | enum | 🔧 provider_config | none/friendly/pragmatic |
| `instructions` | string | 🔧 provider_config | System instructions |
| `developer_instructions` | string | 🔧 provider_config | Developer role message |
| `compact_prompt` | string | 🔧 provider_config | Compaction prompt |
| `tool_output_token_limit` | uint | 🔧 provider_config | |
| `service_tier` | enum | 🔧 provider_config | fast/flex |

#### Approval Policy (granular)
| Field | Ship Status | Notes |
|---|---|---|
| `granular.sandbox_approval` | 🔴 Gap | Shell command approvals |
| `granular.rules` | 🔴 Gap | Execpolicy prompt rules |
| `granular.mcp_elicitations` | 🔴 Gap | MCP input prompts |
| `granular.request_permissions` | 🔴 Gap | Permission requests |
| `granular.skill_approval` | 🔴 Gap | Skill execution approval |

#### MCP Servers
| Feature | Ship Status | Notes |
|---|---|---|
| `[mcp_servers.<name>]` (stdio) | ✅ Compiled | command/args/env |
| `[mcp_servers.<name>]` (HTTP) | ✅ Compiled | url field |
| `cwd` | 🔴 Gap | Working directory |
| `startup_timeout_sec` / `startup_timeout_ms` | 🔴 Gap | Per-server startup timeout |
| `tool_timeout_sec` | 🔴 Gap | Per-tool timeout |
| `enabled` | 🔴 Gap | Server enable/disable |
| `enabled_tools` / `disabled_tools` | 🔴 Gap | Per-server tool allow/block |
| `required` | 🔴 Gap | Fail-fast if server unavailable |
| `bearer_token` / `bearer_token_env_var` | 🔴 Gap | Auth token |
| `oauth_resource` / `scopes` | 🔧 provider_config | OAuth config |
| `env_vars` | 🔴 Gap | Env var names to pass through |
| `env_http_headers` / `http_headers` | 🔴 Gap | HTTP headers (static + env-sourced) |
| `mcp_oauth_callback_port` | 🔧 provider_config | |
| `mcp_oauth_callback_url` | 🔧 provider_config | |
| `mcp_oauth_credentials_store` | 🔧 provider_config | auto/file/keyring |

#### Permissions
| Field | Ship Status | Notes |
|---|---|---|
| `default_permissions` | 🔴 Gap | Named permissions profile |
| `permissions` profiles | 🔴 Gap | Per-profile filesystem + network ACLs |
| `permissions.*.filesystem` | 🔴 Gap | Read/write/none per path |
| `permissions.*.network.mode` | 🔴 Gap | limited/full |
| `permissions.*.network.allowed_domains` | 🔴 Gap | Domain allowlist |
| `permissions.*.network.denied_domains` | 🔴 Gap | Domain denylist |

#### Execution Policy Rules (Starlark)
| Feature | Ship Status | Notes |
|---|---|---|
| `~/.codex/rules/*.rules` | 🔴 Gap | Starlark `prefix_rule()` files |
| `prefix_rule(pattern, decision, justification)` | 🗺️ Maps to permissions | allow/prompt/forbidden per command prefix |
| Admin-enforced rules (`requirements.toml`) | ⬜ Managed only | |
| Feature flags (50+ booleans) | 🔧 provider_config | `features.*` toggles |

#### Multi-Agent
| Feature | Ship Status | Notes |
|---|---|---|
| `agents.max_threads` | 🔴 Gap | Parallel agent limit |
| `agents.max_depth` | 🔴 Gap | Nesting depth |
| `agents.job_max_runtime_seconds` | 🔴 Gap | Per-job timeout |
| Agent role entries (`.config_file`, `.description`) | 🔴 Gap | Role configs |
| `agents/*.toml` files | 🔴 Gap | Role-specific configs |
| `approvals_reviewer` | 🔧 provider_config | user/guardian_subagent |

#### Skills
| Feature | Ship Status | Notes |
|---|---|---|
| `.agents/skills/*/SKILL.md` | ✅ Compiled | Standard skill format |
| `skills.bundled.enabled` | 🔧 provider_config | Toggle bundled skills |
| `skills.config[]` | 🔧 provider_config | Per-skill enable/path override |

#### Plugins & Apps
| Feature | Ship Status | Notes |
|---|---|---|
| `plugins` | 🔴 Gap | Plugin enable/disable map |
| `apps._default` | 🔧 provider_config | Default app policy |
| `apps.<name>` | 🔧 provider_config | Per-app tool approval modes |

#### Environment
| Field | Ship Status | Notes |
|---|---|---|
| `shell_environment_policy.inherit` | 🔧 provider_config | core/all/none |
| `shell_environment_policy.set` | 🔴 Gap | Explicit env var overrides |
| `shell_environment_policy.include_only` / `exclude` | 🔧 provider_config | Regex filters |
| `openai_base_url` | 🔧 provider_config | |
| `chatgpt_base_url` | 🔧 provider_config | |

#### Memories
| Field | Ship Status | Notes |
|---|---|---|
| `memories.generate_memories` | 🔧 provider_config | Auto-generate memories |
| `memories.use_memories` | 🔧 provider_config | Use memories in context |
| `memories.extract_model` / `consolidation_model` | 🔧 provider_config | |
| `memories.max_*` / `no_memories_if_mcp_or_web_search` | 🔧 provider_config | Tuning knobs |

#### Enterprise / Admin
| Feature | Ship Status | Notes |
|---|---|---|
| `/etc/codex/managed_config.toml` | ⬜ Managed only | Cannot be overridden |
| `requirements.toml` | ⬜ Managed only | Allowed policies, sandbox modes, rules |
| `projects.<name>.trust_level` | 🔧 provider_config | trusted/untrusted per project |

#### Observability
| Field | Ship Status | Notes |
|---|---|---|
| `otel.*` | 🔧 provider_config | Full OTEL config |
| `notify` | 🔧 provider_config | External notification command (array) |
| `log_dir` | 🔧 provider_config | |
| `commit_attribution` | 🔧 provider_config | Commit message attribution |

#### UX / Experimental (out of scope)
Fields: `tui.*` (theme, animations, tooltips, alt_screen), `audio.*`, `realtime.*`, `ghost_snapshot.*`, `web_search`, `history.*`, `file_opener`, `check_for_update_on_startup`, `disable_paste_burst`, `hide_agent_reasoning`

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
├── commands/*.md                       # global slash commands (plain Markdown)
├── skills/*/SKILL.md                   # global skills
└── state.vscdb                         # SQLite (all AI settings, models, API keys)

<project>/
├── .cursor/
│   ├── mcp.json                        # project MCP servers
│   ├── rules/*.mdc                     # rules (MDC format with YAML frontmatter)
│   ├── hooks.json                      # project hooks
│   ├── cli.json                        # project CLI permissions
│   ├── commands/*.md                   # project slash commands (plain Markdown)
│   ├── skills/*/SKILL.md              # project skills
│   ├── environment.json                # cloud agent environment config
│   └── Dockerfile                      # custom base image for remote env
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
| `beforeSubmitPrompt` | `UserPromptSubmit` | 🔴 Gap | Blocks before prompt sent to model; stdin includes prompt content |
| `beforeShellExecution` | `PreToolUse` | ✅ Compiled | Can return allow/deny/ask |
| `beforeMCPExecution` | `PreToolUse` | ✅ Compiled | stdin: server, tool_name, tool_input |
| `afterShellExecution` | `PostToolUse` | ✅ Compiled | |
| `afterMCPExecution` | `PostToolUse` | ✅ Compiled | |
| `afterFileEdit` | — | 🔴 Gap | Post-edit formatting/staging; receives old+new content |
| `beforeReadFile` | — | 🔴 Gap | Can *rewrite* file content before model sees it |
| `stop` | `Stop` | ✅ Compiled | |

Hook communication: JSON on stdin/stdout. Common fields: `conversation_id`, `generation_id`, `hook_event_name`, `workspace_roots`.
Scoping: project (`.cursor/hooks.json`) + global (`~/.cursor/hooks.json`). Both layers run.

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

#### Slash Commands (`.cursor/commands/*.md`)
| Feature | Ship Status | Notes |
|---|---|---|
| `.cursor/commands/*.md` | 🗺️ Maps to skills | Project commands (plain Markdown, no frontmatter) |
| `~/.cursor/commands/*.md` | 🗺️ Maps to skills | Global commands |
| `/command-name` invocation | 🗺️ Maps to skills | Filename minus `.md` becomes `/` command |

#### Cloud Agents
| Feature | Ship Status | Notes |
|---|---|---|
| `environment.json` | 🔴 Gap | Cloud agent env config |
| `environment.json` fields | 🔴 Gap | `build.context`, `build.dockerfile`, `install`, `start`, `terminals[]` |
| `.cursor/Dockerfile` | 🔴 Gap | Custom base image for remote env |
| Background agents | ⬜ | Cloud compute (up to 8 parallel via worktrees) |
| Shadow workspace | ⬜ | Background verification |

#### Sandbox
| Feature | Ship Status | Notes |
|---|---|---|
| Auto-Run Mode | 🔴 Gap (SQLite) | Ask Every Time / Sandbox / Run Everything |
| Sandbox policy (macOS Seatbelt) | ⬜ | Dynamic from workspace settings |
| Sandbox policy (Linux Landlock+seccomp) | ⬜ | |
| Command allowlist | ⬜ | Partially broken in Cursor 2.0+ |

---

## Cross-Provider Gap Summary

### Critical Gaps (affect user migration from provider → Ship)

| Gap | Providers Affected | Ship Concept Mapping | Priority |
|---|---|---|---|
| **`provider_config` passthrough** | ALL | Free-form per-provider settings blob | **P0** |
| **Environment variables** | Claude (`env`), Gemini (`.env`), Codex (shell policy) | Ship `[env]` section in preset? | **P0** |
| **Model selection** | Gemini, Codex, Cursor | Already in preset `[profile]` but only compiled for Claude | **P1** |
| **Approval mode** | Gemini (`defaultApprovalMode`), Codex (`approval_policy`) | Maps to Ship workspace modes | **P1** |
| **Sandbox mode** | Codex (`sandbox_mode` + `sandbox_workspace_write` with writable_roots/network), Gemini (`tools.sandbox` + Dockerfile) | Ship concept TBD | **P2** |
| **Codex permission profiles** | Codex (filesystem + network ACLs per profile, Starlark `.rules` files) | Partially maps to Ship permissions but much richer (per-path fs access, domain-level network ACLs) | **P2** |
| **Codex granular approval** | Codex (`approval_policy.granular` — per-category: sandbox, rules, MCP, permissions, skills) | No Ship equivalent | **P2** |
| **MCP per-server config** | Codex (timeouts, enabled), Gemini (timeout, cwd, includeTools, excludeTools, headers, trust), Claude (enable/disable lists) | Extend Ship MCP server TOML | **P2** |
| **Hook triggers (12+ new)** | Claude (12 new), Gemini (6 new), Cursor (3 new: `beforeSubmitPrompt`, `afterFileEdit`, `beforeReadFile`) | Expand Ship hook trigger enum | **P2** |
| **Hook types (3 new)** | Claude (`prompt`, `agent`, `http`) | Ship hook type enum | **P2** |
| **Multi-agent roles** | Codex (`agents`, `*.toml`), Claude (`.claude/agents/`) | Ship team/agent concept | **P2** |
| **Plugins** | Claude (`enabledPlugins`), Codex (`plugins`) | Ship plugins manifest (partial) | **P2** |
| **Slash commands** | Gemini (`.toml` commands with templates), Cursor (`.md` commands) | Maps to Ship skills; Gemini template syntax (`@{}`, `!{}`, `{{args}}`) not supported | **P2** |
| **Tool filtering per MCP** | Gemini (`includeTools`/`excludeTools`), Codex (per-server `enabled`) | Ship MCP server config | **P2** |
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

**What's actually true:** Ship handles the *structural* config well (MCP, rules, skills, permissions, basic hooks) but misses most *behavioral* config (model selection, approval modes, sandbox, env vars, timeouts, plugins). A user migrating a mature Claude Code setup would lose ~75% of their settings.json fields. A Codex user would lose approval_policy, sandbox_mode, permission profiles (fs+network ACLs), Starlark execution rules, and granular approval — arguably their most important settings. Codex's permission model is actually richer than we assumed — it has per-path filesystem access modes, domain-level network ACLs, and Starlark rules for command-level policy.

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
