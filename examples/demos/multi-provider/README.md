# Story: Multi-Provider Setup

**Persona**: Jordan, a developer who uses different AI clients for different
tasks — Gemini for planning/research, Claude for implementation, Codex for
code review and refactoring.

**Arc**: Project setup → provider detection → mode-per-workflow configuration
→ exporting config to all three clients → day-in-the-life mode switching.

## What this demonstrates

- `ship providers detect` — auto-discover installed AI clients
- `ship providers connect/disconnect` — per-project provider management
- `ship mode add` — creating modes scoped to specific workflows
- `ship mcp export <provider>` — exporting Ship config to each client
- The mental model: **modes are provider-agnostic**. The same mode config
  (MCP servers, skills, tool restrictions) gets translated for whichever
  provider you're using.

## Key insight

Modes and providers are **orthogonal**. A mode defines _what_ your agent can
do (capabilities, context, tools). The provider defines _who_ executes it.
You can switch providers without losing your capability configuration, and you
can switch modes without re-configuring your providers.

## Run it

```bash
bash examples/demos/multi-provider/story.sh
```
