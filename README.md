# Ship (Alpha)

Ship is a local-first project memory and execution tool for software teams and AI agents.

For alpha, the focus is one loop:

`Vision -> Release -> Feature -> Spec -> Issues -> ADRs -> Close Feature -> Ship Release`

## Alpha Scope

- Markdown documents with TOML frontmatter for features, issues, specs, and ADRs
- Local `.ship/` project state (no accounts, no cloud dependency)
- CLI for project setup and issue/ADR workflows
- Append-only event stream (`.ship/events.ndjson`) for cross-surface sync
- MCP server over stdio (`ship mcp`) for agent access to project context
- Tauri UI under active development

## Quick Start

Initialize Ship in a repo:

```bash
ship init
```

List issues:

```bash
ship issue list
```

Create an issue:

```bash
ship issue create "Implement Kanban drag and drop" "Enable moving issue cards across columns."
```

Start MCP server (stdio):

```bash
ship mcp
```

## Core CLI Commands

```bash
ship init
ship issue create <title> <description>
ship issue list
ship issue move <file_name> <from_status> <to_status>
ship issue note <file_name> <note>
ship adr create <title>
ship spec create <title>
ship spec list
ship release create <version>
ship release list
ship feature create <title>
ship feature list
ship event list --since 0 --limit 50
ship event ingest
ship projects
ship mcp
ship config
```

Run `ship --help` for the full command set.

## Project Structure

```text
.ship/
├── config.toml
├── templates/
│   ├── RELEASE.md
│   ├── FEATURE.md
│   ├── ISSUE.md
│   ├── SPEC.md
│   ├── VISION.md
│   └── ADR.md
├── releases/
├── features/
├── issues/
│   ├── backlog/
│   ├── in-progress/
│   ├── review/
│   ├── done/
│   └── blocked/
├── specs/
│   └── vision.md
├── adrs/
├── log.md
└── events.ndjson
```

All `.ship` paths are lowercase.

Default git policy is opinionated for alpha:

- committed: `releases`, `features`, `specs`, `adrs`, `config.toml`, `templates`
- local-only: `issues`, `log.md`, `events.ndjson`, `plugins`

## UI Development

From `crates/ui`:

```bash
pnpm install
pnpm build
pnpm dev
```

## Example Workspace

Use [`example/projects-e2e/`](./example/projects-e2e/) to validate project workflows end-to-end without committing generated `.ship/` state.

## Notes

- This repo is in alpha and evolves quickly.
- Source-of-truth product direction lives in `.ship/specs/alpha-spec.md`.
