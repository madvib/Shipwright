---
name: drizzle-studio
stable-id: drizzle-studio
description: Use when you need to browse, query, or edit your database visually. Starts Drizzle Studio and renders it in Ship Studio's session.
tags: [database, drizzle, orm, dev-tools]
authors: [ship]
artifacts: [url]
---

# Drizzle Studio

Starts Drizzle Studio and renders the database browser in the session canvas.

## Start

```bash
SESSION_ROOT="$(git rev-parse --show-toplevel)/.ship-session"
URL="http://localhost:{{ port }}"

if curl -sf "$URL" > /dev/null 2>&1; then
  echo "Drizzle Studio already running on port {{ port }}"
else
  npx drizzle-kit studio --port {{ port }} --host {{ host }}{% if config != "drizzle.config.ts" %} --config {{ config }}{% endif %} &
  for i in $(seq 1 15); do
    curl -sf "$URL" > /dev/null 2>&1 && break || sleep 1
  done
fi

echo "$URL" > "$SESSION_ROOT/{{ stable_id }}.url"
```

> Default port `{{ port }}` matches the container port mapping. Set `host` to `0.0.0.0` inside Docker/Podman.

{% if dialect == "sqlite" %}
Drizzle Studio connects to the SQLite file specified in your `{{ config }}`. Make sure the database file exists before starting.
{% elif dialect == "postgres" %}
Drizzle Studio connects to Postgres using the `DATABASE_URL` in your environment. Ensure the database is running and the env var is set.
{% elif dialect == "mysql" %}
Drizzle Studio connects to MySQL using the connection string in your `{{ config }}`. Ensure the database is running.
{% endif %}

## Use

- Browse tables in the left sidebar
- Click a row to edit values inline
- Run raw SQL from the Query tab
- Changes take effect immediately — no migration needed for data edits

## Stop

```bash
pkill -f "drizzle-kit.*studio"
rm -f "$(git rev-parse --show-toplevel)/.ship-session/{{ stable_id }}.url"
```
