---
name: commit
stable-id: commit
description: Use when committing changes. Formats commit messages, applies signing, and attributes co-authors based on per-user and per-project settings.
tags: [git, workflow]
authors: [ship]
---

# Commit

Write commit messages that match `{{ commit_style }}`.

{% if commit_style == "conventional" %}
## Conventional Commits

Format: `<type>(<scope>): <subject>`

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `perf`, `ci`

- Subject is imperative, lowercase, no period
- Body explains the *why*, not the *what*
- Breaking changes: add `!` after type or `BREAKING CHANGE:` footer

```
feat(auth): add OAuth2 PKCE flow

Replaces the implicit grant flow, which is deprecated in OAuth 2.1.
```

{% elif commit_style == "gitmoji" %}
## Gitmoji

Start every commit with the appropriate emoji:

| Emoji | Use |
|-------|-----|
| ✨ | New feature |
| 🐛 | Bug fix |
| ♻️ | Refactor |
| ✅ | Tests |
| 📝 | Docs |
| 🔧 | Config/chore |
| 🚀 | Performance |

```
✨ add OAuth2 PKCE flow
```

{% elif commit_style == "angular" %}
## Angular Commit Style

Format: `<type>(<scope>): <subject>`

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`

Follow the Angular commit message guidelines strictly.

{% endif %}

## Rules

- Stage explicit files only — never `git add .` or `git add -A`
- One logical change per commit
- Never skip hooks (`--no-verify`)
- Never amend published commits
{% if sign_commits %}
- Sign every commit with `-S`
{% endif %}
{% if co_authors %}
- Append these trailers to every commit body:
{% for author in co_authors %}
  `Co-Authored-By: {{ author }}`
{% endfor %}
{% endif %}
