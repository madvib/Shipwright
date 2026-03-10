# Hook Mapping Guide

Use this table when converting Ship triggers to provider-native events.

| Ship trigger | Claude event | Gemini event | Notes |
|---|---|---|---|
| `SessionStart` | `SessionStart` | `SessionStart` | Good for context injection |
| `UserPromptSubmit` | `UserPromptSubmit` | n/a | Claude-only |
| `PreToolUse` | `PreToolUse` | `BeforeTool` | Safe cross-provider mapping |
| `PostToolUse` | `PostToolUse` | `AfterTool` | Safe cross-provider mapping |
| `Stop` | `Stop` | `SessionEnd` | End-of-session mapping |
| `Notification` | `Notification` | `Notification` | Telemetry/event forwarding |
| `PreCompact` | `PreCompact` | `PreCompress` | Naming differs by provider |

## Export Shape Examples

### Claude

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "ship hooks run",
            "timeout": 2000,
            "description": "policy gate"
          }
        ]
      }
    ]
  }
}
```

### Gemini

```json
{
  "hooks": {
    "BeforeTool": [
      {
        "matcher": "run_shell_command",
        "hooks": [
          {
            "name": "before-tool-guard",
            "type": "command",
            "command": "ship hooks run",
            "timeout": 1200,
            "description": "decompose chained shell command"
          }
        ]
      }
    ]
  }
}
```

## Guardrail

If a provider lacks native hooks support, do not invent a pseudo-field in exported config.
Keep hooks in Ship state and surface unsupported status in UI.

## Runtime Notes

- Hooks UI is intentionally hidden for now; Ship manages the baseline internally.
- Ship writes hook runtime policy artifacts to `.ship/agents/runtime/`:
  - `hook-context.md`
  - `envelope.json`
- Hook telemetry is internal and global at:
  - `~/.ship/state/telemetry/hooks/events.ndjson`
