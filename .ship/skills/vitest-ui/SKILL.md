---
name: vitest-ui
stable-id: vitest-ui
description: Use when you want to run, filter, and watch tests visually. Starts Vitest's interactive UI and renders it live in Ship Studio's session.
tags: [testing, vitest, ui, dev-tools]
authors: [ship]
artifacts: [url]
compatibility: Requires vitest with @vitest/ui. Optional @vitest/coverage-v8 for coverage tab.
---

# Vitest UI

Starts Vitest's interactive test runner and surfaces it in the session canvas.

## Start

Check if already running, then start only if needed:

```bash
SESSION_ROOT="$(git rev-parse --show-toplevel)/.ship-session"
URL="http://localhost:{{ port }}/__vitest__/"

if curl -sf "$URL" > /dev/null 2>&1; then
  echo "Vitest UI already running on port {{ port }}"
else
  pnpm exec vitest --ui --api.port={{ port }} --api.host={{ host }}{% if filter %} {{ filter }}{% endif %}{% if coverage %} --coverage{% endif %} &
  for i in $(seq 1 15); do
    curl -sf "$URL" > /dev/null 2>&1 && break || sleep 1
  done
fi

echo "$URL" > "$SESSION_ROOT/{{ stable_id }}.url"
```

> Default port `{{ port }}` matches the container port mapping. Set `host` to `0.0.0.0` inside Docker/Podman.

## Stop

```bash
pkill -f "vitest.*{{ port }}"
rm -f "$(git rev-parse --show-toplevel)/.ship-session/{{ stable_id }}.url"
```

## Filter tests

Pass a pattern to scope the run, or use the search box in the UI:

```bash
pnpm exec vitest --ui --api.port={{ port }} --api.host={{ host }} src/auth &
```

{% if coverage %}
## Coverage

Coverage tab is enabled via the `coverage` var. Requires `@vitest/coverage-v8`:

```bash
pnpm add -D @vitest/coverage-v8
```
{% endif %}
