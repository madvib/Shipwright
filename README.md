# ship-cli

AI-assisted project tracking and feature development CLI.

Feature ideas:
I want to use MCP sampling to allow generation inside the UI. 

I want to be able to brainstorm about new issues, query a code base etc. 

I think a time tracking plugin would be sweet.

We need better handling for multiple projects.

The global config needs more thought.

I was hoping to use markdown over json for issues.

Better editor in the UI. 

Customizable issue categories.

Drag and drop.

Ephemeral notes.

Draw.io or similar integration

Auto-ADR Suggestions: The agent could watch your files and prompt you: "I see you're adding a second database, should we create an ADR for this architecture change?"
Ghost Issues: Automatically scan TODOs and FIXME's in your code and surface them as "suggested issues" in the UI.
Sync Context: Linking specific code symbols directly to issues, so when you open an issue, it automatically opens the relevant files in the background (contextual awareness).

## Features



- **Project Tracking**: Manage Issues and Architecture Decision Records (ADRs) directly in your repository under the `.ship/` directory.
- **MCP Server**: Built-in Model Context Protocol server for AI agents to interact with your project state.
- **Web Dashboard**: Visual dashboard to view project progress, ADRs, and logs.
- **Customizable Templates**: Eject and customize Markdown templates for Issues and ADRs.
- **Agent Logging**: Every action performed by an agent via MCP is logged for transparency.

## Installation

```bash
npm install -g ship-cli
```

## Quick Start

Initialize project tracking in your repo:

```bash
ship project init
```

Start the Web UI:

```bash
ship project ui
```

Start the MCP server:

```bash
ship project mcp
```

## Commands

- `ship issue create <title>`: Create a new issue.
- `ship issue move <file> <from> <to>`: Move issue status.
- `ship adr create <title>`: Create a new ADR.
- `ship project link <source> <target>`: Link two items together.
- `ship project eject-templates`: Customize templates.

## Directory Structure

```
.ship/
├── ADR/           # Architecture Decision Records
├── Issues/        # Issues categorized by status
│   ├── backlog/
│   ├── in-progress/
│   ├── done/
│   └── blocked/
├── templates/     # Customizable Markdown templates
├── log.md         # History of agent actions
└── README.md      # Project tracking overview
```
