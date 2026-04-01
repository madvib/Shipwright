---
name: vite-inspect
stable-id: vite-inspect
description: Use when you need to understand your Vite bundle — inspect plugins, module graph, transform chains, and build performance. Renders vite-plugin-inspect's UI in Ship Studio's session.
tags: [vite, bundle, performance, dev-tools]
authors: [ship]
artifacts: [url]
---

# Vite Inspect

Surfaces the Vite plugin inspection UI in the session canvas. Shows each module's transform chain, which plugins touched it, and how long each transform took.

## Setup

Add `vite-plugin-inspect` to your project (one-time):

```bash
{% if package_manager == "pnpm" %}pnpm add -D vite-plugin-inspect
{% elif package_manager == "yarn" %}yarn add -D vite-plugin-inspect
{% else %}npm install -D vite-plugin-inspect
{% endif %}
```

Add to `vite.config.ts`:

```ts
import Inspect from 'vite-plugin-inspect'

export default {
  plugins: [
    Inspect(),  // adds /__inspect/ route to dev server
  ],
}
```

## Start

```bash
SESSION_ROOT="$(git rev-parse --show-toplevel)/.ship-session"
URL="http://localhost:{{ port }}/__inspect/"

if curl -sf "$URL" > /dev/null 2>&1; then
  echo "Vite dev server already running on port {{ port }}"
else
{% if package_manager == "pnpm" %}  pnpm dev --port {{ port }} --host {{ host }} &
{% elif package_manager == "yarn" %}  yarn dev --port {{ port }} --host {{ host }} &
{% else %}  npm run dev -- --port {{ port }} --host {{ host }} &
{% endif %}  for i in $(seq 1 15); do
    curl -sf "$URL" > /dev/null 2>&1 && break || sleep 1
  done
fi

echo "$URL" > "$SESSION_ROOT/{{ stable_id }}.url"
```

> Default port `{{ port }}` matches the container port mapping. Set `host` to `0.0.0.0` inside Docker/Podman.

## Use

- **Module graph** — see all modules and their import relationships
- **Plugin transforms** — click any module to see which plugins transformed it and in what order
- **Timing** — spot slow transforms that hurt HMR or build times
- **Resolved IDs** — see how imports resolve to actual file paths

## Stop

```bash
pkill -f "vite.*{{ port }}"
rm -f "$(git rev-parse --show-toplevel)/.ship-session/{{ stable_id }}.url"
```
