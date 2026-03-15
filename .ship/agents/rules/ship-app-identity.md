# Ship App Identity

* This repository is the Ship platform: a compiler, workspace runtime, and workflow substrate for AI agents.

* The current product surface is Ship Studio — a web-first compiler tool at apps/web/. This is the active development target.

* Architecture is defined in ARCHITECTURE.md at the repo root. Read it before making structural decisions.

* Platform/workflow separation is strict: workspace, session, event, preset, skill, MCP, permission are platform primitives. Feature, release, issue, spec, vision are workflow-layer types (shipflow). Do not add workflow types to platform code.

* CLI and MCP are transport layers over runtime operations. Business logic belongs in the runtime.

* apps/desktop/ is frozen — do not invest in new features there without explicit instruction.

* apps/site/ is archived — superseded by apps/web/.
