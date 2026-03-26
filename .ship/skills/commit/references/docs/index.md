---
title: Commit skill
description: Formats commit messages, applies signing, and attributes co-authors based on per-user and per-project settings.
---

# Commit

The commit skill guides the agent when writing git commit messages. It adapts the format, signing, and attribution to your settings — the same skill produces different output for each user and project.

## What it does

- Writes commit messages in your preferred format (`conventional`, `gitmoji`, or `angular`)
- Optionally signs commits with GPG or SSH (`-S`)
- Appends `Co-Authored-By` trailers for each configured co-author
- Enforces rules: explicit file staging, one logical change per commit, no hook bypasses

## Variables

| Variable | Type | Scope | Default | Description |
|----------|------|-------|---------|-------------|
| `commit_style` | enum | global | `conventional` | Format applied to every commit message. `conventional` = `feat:`/`fix:` prefixes, `gitmoji` = emoji prefixes, `angular` = Angular-style. |
| `sign_commits` | bool | project | `false` | Whether to pass `-S` to `git commit`. Requires GPG or SSH signing to be configured. |
| `co_authors` | array | project | `[]` | List of `Co-Authored-By` trailers appended to every commit. Each entry: `"Name <email>"`. |

### Scope meanings

- **global** — machine-wide, follows you across all projects
- **project** — per working directory, intended to be shared with the team (committed in `.ship/`)
- **local** — per working directory, personal override, not shared

## Setting variables

```bash
# Set your preferred commit style (global — applies everywhere)
ship vars set commit commit_style gitmoji

# Enable commit signing for this project
ship vars set commit sign_commits true

# Add a co-author (appended, not replaced)
ship vars append commit co_authors "Alice <alice@example.com>"

# See the current merged state for this context
ship vars get commit

# Reset all vars to defaults
ship vars reset commit
```

## Commit styles

### conventional (default)

Format: `<type>(<scope>): <subject>`

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `perf`, `ci`

Rules:
- Subject is imperative, lowercase, no trailing period
- Body explains the *why*, not the *what*
- Breaking changes: add `!` after type or a `BREAKING CHANGE:` footer

```
feat(auth): add OAuth2 PKCE flow

Replaces the implicit grant flow, which is deprecated in OAuth 2.1.
```

```
fix(api): handle null pointer in token refresh
```

```
feat(config)!: rename settings.json to settings.jsonc

BREAKING CHANGE: existing settings.json files must be renamed.
```

### gitmoji

Start every commit subject with the appropriate emoji:

| Emoji | Use |
|-------|-----|
| ✨ | New feature |
| 🐛 | Bug fix |
| ♻️ | Refactor |
| ✅ | Tests |
| 📝 | Docs |
| 🔧 | Config / chore |
| 🚀 | Performance |

```
✨ add OAuth2 PKCE flow
```

### angular

Format: `<type>(<scope>): <subject>`

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`

Follows the Angular commit message guidelines strictly. Suitable for projects that use `@angular/changelog` or similar tooling.

## Signing commits

When `sign_commits` is `true`, the agent passes `-S` to every `git commit` call. This requires your local git to be configured with a GPG or SSH signing key.

```bash
# Verify your signing setup before enabling
git config --global user.signingkey
gpg --list-secret-keys

# Enable for this project
ship vars set commit sign_commits true
```

## Co-authors

Co-authors are appended as `Co-Authored-By` trailers in the commit body. This is the GitHub-recognized format for attributing multiple contributors.

```bash
# Add co-authors one at a time
ship vars append commit co_authors "Alice <alice@example.com>"
ship vars append commit co_authors "Bob <bob@example.com>"
```

Resulting commit body:

```
feat(auth): add OAuth2 PKCE flow

Replaces the implicit grant flow, deprecated in OAuth 2.1.

Co-Authored-By: Alice <alice@example.com>
Co-Authored-By: Bob <bob@example.com>
```

## Rules enforced by the skill

- **Stage explicit files only.** Never `git add .` or `git add -A`. Name each file.
- **One logical change per commit.** If you have unrelated changes, split them.
- **Never skip hooks.** `--no-verify` is not acceptable.
- **Never amend published commits.** Create a new commit instead.

## How Ship resolves the template

`SKILL.md` is a MiniJinja template. When you run `ship use` or `ship compile`, Ship:

1. Reads defaults from `assets/vars.json`
2. Merges global state from `platform.db` KV (`skill_vars:commit`)
3. Merges local state (`skill_vars.local:{ctx}:commit`)
4. Merges project state (`skill_vars.project:{ctx}:commit`)
5. Renders `SKILL.md` with the merged values
6. Writes the resolved output to your provider config (e.g. `CLAUDE.md`)

Undefined variables render as empty string. Template errors fall back to original content with a warning.
