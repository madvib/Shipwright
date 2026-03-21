# Job Autostart

On session start, check for a job spec at `.ship-session/job-spec.md` (preferred) or `job-spec.md` in the working directory. If found, read it immediately.

Then check the `## Mode` field:

- **`autonomous`** — begin executing immediately. No preamble, no confirmation. Log questions via `append_job_log` rather than asking the human.
- **`interactive`** — present your understanding of the spec and your planned approach to the human. Wait for approval before executing.
- **field absent** — treat as `autonomous`.

Do not ask clarifying questions before reading the spec. Everything you need to begin is in the file.
