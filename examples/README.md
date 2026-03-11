# Example Workspaces

Runnable examples and end-to-end validation suites for Ship features.

## Stories

Narrative walkthroughs that demonstrate real-world Ship workflows:

- [`stories/solo-dev/`](./stories/solo-dev/) — solo dev ships a v0.1.0 release with AI assistance
- [`stories/multi-provider/`](./stories/multi-provider/) — dev uses Claude, Codex, and Gemini with mode-per-workflow
- [`stories/team-handoff/`](./stories/team-handoff/) — senior dev hands off to a new contributor via committed project state

```bash
cargo build -p cli
bash examples/stories/solo-dev/story.sh
bash examples/stories/multi-provider/story.sh
bash examples/stories/team-handoff/story.sh
```

## E2E Validation

- `projects-e2e/`: project workflow validation (CLI + MCP + UI) with local `.ship/` state ignored by git.
