+++
id = "clean-code"
name = "Clean Code"
source = "custom"
+++

# Clean Code

Use this skill when implementing or reviewing code changes.

## Rules
- Prefer small, focused functions with one responsibility.
- Remove duplication before adding new branches or flags.
- Name things for intent; avoid abbreviations unless domain-standard.
- Keep modules cohesive and dependencies explicit.
- Keep diffs minimal, but do not leave known dead code nearby.

## Working Checklist
- Can this change be split into smaller units?
- Is there repeated logic that should be extracted?
- Are error states explicit and tested?
- Is the public surface simpler after the change?

## Review Bar
Reject changes that increase coupling, hide side effects, or duplicate logic when extraction is straightforward.