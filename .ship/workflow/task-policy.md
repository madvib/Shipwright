# Vibe Project Task Policy

To ensure consistent project tracking and transparency, all contributors (human or AI) must adhere to the following rules:

1.  **Issue First**: Before starting work on any feature or bug, ensure a corresponding issue exists in `.project/Issues/`. Move it to `in-progress` using the `vibe issue move` command or MCP tool.
2.  **Atomic Task Updates**: After completing a functional unit of work (e.g., a new test file, a major function implementation), update the `## Tasks` section of the active issue file.
3.  **Action Logging**: Use `vibe log_action` (or the MCP equivalent) to record every significant decision or completion milestone.
4.  **Verification**: Always run `npm run test` before finalizing a task to ensure the project remains stable.

## Enforcement Mechanism (Proposed)

To prevent issues from going out of sync, we recommend:
- **Pre-commit Hook**: A script that verifies if the current branch matches an `in-progress` issue in `.project/Issues/in-progress/`.
- **Sync Check**: A `vibe project check` command that flags issues missing recent activity or with incomplete task lists.
