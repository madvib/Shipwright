---
name: storybook
stable-id: storybook
description: Use when developing or reviewing UI components in isolation. Starts Storybook and renders the component browser in Ship Studio's session.
tags: [ui, components, storybook, dev-tools]
authors: [ship]
artifacts: [url]
---

# Storybook

Starts Storybook and surfaces the component browser in the session canvas.

## Start

```bash
SESSION_ROOT="$(git rev-parse --show-toplevel)/.ship-session"
URL="http://localhost:{{ port }}"

if curl -sf "$URL" > /dev/null 2>&1; then
  echo "Storybook already running on port {{ port }}"
else
  npx storybook dev --port {{ port }} --host {{ host }}{% if config_dir != ".storybook" %} --config-dir {{ config_dir }}{% endif %} --ci &
  # First build takes ~10s, subsequent starts use cache
  for i in $(seq 1 30); do
    curl -sf "$URL" > /dev/null 2>&1 && break || sleep 1
  done
fi

echo "$URL" > "$SESSION_ROOT/{{ stable_id }}.url"
```

> Default port `{{ port }}` matches the container port mapping. Set `host` to `0.0.0.0` inside Docker/Podman.

Storybook's first start takes longer — it compiles all stories. Subsequent starts use the cache and are faster.

## Develop a component

1. Open a story file or create one:

```bash
# Component file: src/components/Button/Button.stories.tsx
```

2. Storybook hot-reloads on save — changes appear in the session canvas immediately.

{% if framework == "react" %}
## React story format

```tsx
import type { Meta, StoryObj } from '@storybook/react'
import { Button } from './Button'

const meta: Meta<typeof Button> = { component: Button }
export default meta
type Story = StoryObj<typeof Button>

export const Primary: Story = { args: { label: 'Click me', variant: 'primary' } }
export const Disabled: Story = { args: { label: 'Disabled', disabled: true } }
```
{% elif framework == "vue" %}
## Vue story format

```ts
import type { Meta, StoryObj } from '@storybook/vue3'
import MyButton from './MyButton.vue'

const meta: Meta<typeof MyButton> = { component: MyButton }
export default meta
type Story = StoryObj<typeof MyButton>

export const Primary: Story = { args: { label: 'Click me' } }
```
{% endif %}

## Stop

```bash
pkill -f "storybook.*{{ port }}"
rm -f "$(git rev-parse --show-toplevel)/.ship-session/{{ stable_id }}.url"
```
