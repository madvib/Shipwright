+++
id = "brbWqudi"
title = "AI Integration: Direct CLI Adapter Path"
created = "2026-03-04T03:01:36.184101+00:00"
updated = "2026-03-04T03:01:36.184101+00:00"
tags = []
+++

## Goal\nMake in-app AI actions reliable without API key dependence.\n\n## Scope\n- Replace brittle pass-through path with direct CLI adapter execution\n- Define strict command/response contract\n- Add integration tests with deterministic stubs\n\n## Acceptance\n- AI actions work through CLI adapter end-to-end\n- Failures are surfaced with actionable errors\n