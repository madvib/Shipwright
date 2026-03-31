---
name: mcp-inspector
stable-id: mcp-inspector
description: Use when debugging or exploring an MCP server — inspect tools, resources, prompts, and live call responses. Renders the MCP Inspector UI in Ship Studio's session.
tags: [mcp, debugging, dev-tools, inspector]
authors: [ship]
artifacts: [url]
---

# MCP Inspector

Starts the MCP Inspector and renders it in the session canvas. Connect it to any MCP server to browse tools, fire test calls, and inspect responses live.

## Start

```bash
SESSION_ROOT="$(git rev-parse --show-toplevel)/.ship-session"
URL="http://localhost:{{ port }}"

if curl -sf "$URL" > /dev/null 2>&1; then
  echo "MCP Inspector already running on port {{ port }}"
else
  DANGEROUSLY_OMIT_AUTH=true CLIENT_PORT={{ port }} HOST={{ host }} npx @modelcontextprotocol/inspector &
  for i in $(seq 1 20); do
    curl -sf "$URL" > /dev/null 2>&1 && break || sleep 1
  done
fi

echo "$URL" > "$SESSION_ROOT/{{ stable_id }}.url"
```

`DANGEROUSLY_OMIT_AUTH=true` is required so the plain URL works in the Studio iframe without a session token. `CLIENT_PORT` sets the UI port; the proxy always runs on its default port (6277). `HOST` binds to all interfaces — set to `0.0.0.0` in container environments.

## Connect to a server

In the Inspector UI, enter the server command in the connection panel. Examples:

{% if runtime.agents %}
Configured agents in this project:
{% for a in runtime.agents %}
- **{{ a.id }}**: `ship mcp serve --agent {{ a.id }}`
{% endfor %}
{% endif %}

Or connect directly to any server:

```
Command: npx
Args:    -y @modelcontextprotocol/server-filesystem /path/to/dir
```

## Connect to the Ship runtime

```
Command: ship
Args:    mcp serve
```

This exposes all Ship MCP tools — workspaces, sessions, jobs, skills — for live inspection.

## Stop

```bash
pkill -f "mcp-inspector"
rm -f "$(git rev-parse --show-toplevel)/.ship-session/{{ stable_id }}.url"
```
