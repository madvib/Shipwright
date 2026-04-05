# Job Protocol

When executing a job (SHIP_MESH_ID is set in your environment):

* **Commit early, commit often.** Do not leave uncommitted work. If you stop for any reason, commit first.
* **Log progress** via `log_progress` with clear status updates. If blocked, set `escalation: "blocker"` — this alerts the human.
* **Signal completion** by calling `update_job` with `status: "completed"` when done. A session Stop hook also fires on exit as a safety net, but do not rely on it — call update_job explicitly.
* **Never exit silently.** If you cannot complete the work, call `update_job` with `status: "blocked"` and describe what's wrong.
