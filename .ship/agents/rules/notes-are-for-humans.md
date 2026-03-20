# Notes and ADRs Are For Humans

Notes and ADRs are human-facing documents. They are NOT agent scratch pads, coordination channels, or policy stores.

## What agents MUST NOT do with notes

* Write agent plans, implementation details, or task tracking into notes
* Use notes for cross-agent communication or coordination
* Store agent intent, policy, or source of truth in notes
* Create notes as a way to persist agent context across sessions

## What agents DO with notes

* Help humans draft and refine notes when asked
* Surface existing notes that are relevant to the current conversation
* Read notes for human-authored context and decisions

## Agent state belongs elsewhere

* Session progress → `log_progress` / `append_job_log`
* Plans and specs → `.ship-session/` scratchpad files (gitignored)
* Coordination → job queue (`create_job`, `update_job`)
* Architecture decisions → ADRs (but only when the human is driving the decision)
* Handoffs → `complete_workspace` / handoff.md in worktree
