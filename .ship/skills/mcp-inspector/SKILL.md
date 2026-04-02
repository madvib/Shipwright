---
name: mcp-inspector
stable-id: mcp-inspector
description: Debug and test MCP servers — launch the Inspector UI, connect to any server (stdio or HTTP), inspect tools/resources/prompts, watch SSE notifications live. Renders in Ship Studio.
tags: [mcp, debugging, dev-tools, inspector]
authors: [ship]
artifacts: [url]
---

# MCP Inspector

Debug MCP servers interactively. Launch the Inspector, connect it to any server, fire tool calls, and watch the notification stream in real time.

## Start

```bash
SESSION_ROOT="$(git rev-parse --show-toplevel)/.ship-session"
URL="http://{{ host }}:{{ port }}"

if curl -sf "$URL" > /dev/null 2>&1; then
  echo "MCP Inspector already running on port {{ port }}"
else
  DANGEROUSLY_OMIT_AUTH=true CLIENT_PORT={{ port }} HOST={{ host }} npx -y @modelcontextprotocol/inspector &
  for i in $(seq 1 20); do
    curl -sf "$URL" > /dev/null 2>&1 && break || sleep 1
  done
fi

echo "$URL" > "$SESSION_ROOT/{{ stable_id }}.url"
```

`DANGEROUSLY_OMIT_AUTH=true` lets the URL work in Studio's iframe without a token. `CLIENT_PORT` sets the UI port. The proxy runs on 6277 by default.

## Stop

```bash
pkill -f "mcp-inspector"
rm -f "$(git rev-parse --show-toplevel)/.ship-session/{{ stable_id }}.url"
```

## Quick-connect targets

### Ship runtime (stdio)

```
Transport: STDIO
Command:   ship
Args:      mcp serve
```

Exposes all Ship MCP tools — workspaces, sessions, jobs, skills.

### Ship network daemon (Streamable HTTP)

```
Transport: Streamable HTTP
URL:       http://127.0.0.1:9315/mcp
```

Exposes mesh tools — register, send, broadcast, discover, inbox. Use this to debug cross-agent messaging and SSE notification delivery.

{% if runtime.agents %}
### Project agents

{% for a in runtime.agents %}
- **{{ a.id }}**: `ship mcp serve --agent {{ a.id }}`
{% endfor %}
{% endif %}

### Any server

```
Transport: STDIO
Command:   npx
Args:      -y <package-name> <args>
```

## Debugging workflow

### Phase 1: Verify connectivity

1. Open the Inspector UI in Studio or browser
2. Connect to the target server
3. Confirm the **Tools**, **Resources**, and **Prompts** tabs populate
4. Check the **Notifications** pane is empty and alive (no errors)

If connection fails: check the server process is running, the port is correct, and the transport matches (stdio vs HTTP).

### Phase 2: Test tool calls

1. Pick a tool from the **Tools** tab
2. Fill in parameters and execute
3. Verify the response shape and content
4. Check the **Notifications** pane for any server-sent notifications triggered by the call

### Phase 3: Debug notifications (SSE)

This is the critical path for ship-network and Studio annotations.

1. Connect to the Streamable HTTP server
2. Call `mesh_register` to join the mesh
3. From a second client (curl, another Inspector, or another agent), send a message targeting the registered agent
4. Watch the **Notifications** pane — `ship/event` notifications should appear here
5. If notifications don't appear, the SSE stream is broken

**Diagnosing SSE failures:**

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| Tool calls work, notifications never arrive | Client not holding GET SSE stream | Check transport config — Inspector opens SSE automatically |
| Notification pane shows errors | `peer.send_notification` failing | Check daemon logs at `/tmp/ship-network.log` |
| Notification arrives in Inspector but not Claude Code | Claude Code doesn't open GET SSE for HTTP servers | Use `mesh_inbox` polling as fallback |
| "Unauthorized: Session not found" on GET | Session expired between requests | Re-initialize — check session management |

### Phase 4: Inspect the wire protocol

Use the Inspector's request/response log to see raw JSON-RPC messages:

```
→ {"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"mesh_send",...}}
← {"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"ok: routed mesh.send"}]}}
← {"jsonrpc":"2.0","method":"notifications/message","params":{"method":"ship/event","params":{...}}}
```

The third line is the push notification. If it appears in the Inspector but not in Claude Code, the issue is client-side.

## curl cheatsheet for Streamable HTTP

When you need to test outside the Inspector:

```bash
# Initialize a session
curl -s -X POST http://127.0.0.1:9315/mcp \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -D /tmp/mcp-headers.txt \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"debug","version":"1.0.0"}}}'

# Extract session ID
SESSION=$(grep -i mcp-session-id /tmp/mcp-headers.txt | awk '{print $2}' | tr -d '\r')

# Send initialized notification
curl -s -X POST http://127.0.0.1:9315/mcp \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -H "Mcp-Session-Id: $SESSION" \
  -d '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}'

# Call a tool
curl -s -X POST http://127.0.0.1:9315/mcp \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -H "Mcp-Session-Id: $SESSION" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"mesh_register","arguments":{"agent_id":"debug.probe","capabilities":[]}}}'

# Open SSE stream for push notifications (hold open)
curl -s -N -X GET http://127.0.0.1:9315/mcp \
  -H "Accept: text/event-stream" \
  -H "Mcp-Session-Id: $SESSION"
```

## Key files

| File | Purpose |
|------|---------|
| `apps/ship-network/src/server.rs` | NetworkServer MCP tools |
| `apps/ship-network/src/connections.rs` | EventRelay, McpEventSink, Inbox |
| `apps/ship-network/src/handler.rs` | ServerHandler — `on_initialized` stores Peer |
| `apps/mcp/src/studio_server.rs` | Studio MCP server (stdio) |
