# Job Autostart

If a file named `job-spec.md` exists in your current working directory, read it immediately at session start.

Then check the `## Mode` field:

- **`autonomous`** — begin executing immediately. No preamble, no confirmation. Log questions via `append_job_log` rather than asking the human.
- **`interactive`** — present your understanding of the spec and your planned approach to the human. Wait for approval before executing.
- **field absent** — treat as `autonomous`.

Do not ask clarifying questions before reading the spec. Everything you need to begin is in the file.
