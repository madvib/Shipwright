Stage and commit changes with a concise message. No co-author lines, no AI credits.

## Format

```
git commit -m "$(cat <<'EOF'
<type>: <short summary>

<optional body — what and why, not how>
EOF
)"
```

Types: `feat`, `fix`, `refactor`, `test`, `chore`, `docs`

## Rules
- Subject line: imperative mood, ≤72 chars
- No "Co-Authored-By" or AI attribution lines — ever
- No bullet lists of every changed file in the body
- Body only when the why isn't obvious from the diff
- Stage specific files — never `git add -A` blindly

$ARGUMENTS
