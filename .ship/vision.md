+++
title = "Vision"
updated = "2026-03-11T00:00:00Z"
+++

# Ship — Vision

---

## The Problem

Software drifts from intent. Not because developers stop caring, but because intent is stored in heads and conversations and closed browser tabs — not in the system itself. Every decision made in a meeting, every constraint discovered at 2am, every "we tried that and here's why it doesn't work" — most of it evaporates.

Agents accelerate this. An agent session starts cold. It has no model of what the system is supposed to be, what shaped it, or what it's drifting away from. You compensate by pasting context into every prompt. The agent executes. The system moves. Intent erodes a little more.

The tools aren't the problem. The missing layer is structured, persistent, machine-readable intent — and a mechanism that continuously closes the gap between what the system declares it should be and what it actually is.

---

## The Belief

Declarative systems win.

Terraform over bash scripts. Kubernetes over manual deploys. In every domain where the principle has been applied, the same thing happened: teams stopped describing *steps* and started describing *desired state*. The system took responsibility for closing the gap.

Software development is the last domain where this hasn't happened. We still describe steps — tickets, tasks, issues — and call that project management. We have no primitive for declaring what the system should *be* and verifying that it still is.

Ship is that primitive.

A feature is not a task. It is a declaration of desired system state — what the software should do, how it should behave, what the acceptance criteria are. Tests are sensors that measure the gap between declared and actual. Documentation reflects runtime behavior automatically. After every agent session, hooks close the drift. The system always knows what it is supposed to be.

---

## The Model

```
Vision (1)
└── Capability Map
    └── Capability (N)
        └── Feature (N)                  ← declared desired state
            ├── Declaration              ← human-authored contract
            ├── Status                   ← actual state (tests, coverage, runtime checks)
            ├── Delta                    ← the gap — computed, not authored
            └── Docs                     ← derived from declaration + session records + test outcomes
                └── Workspace (1)
                    └── Session (N)
                        └── Session Record   ← immutable audit artifact
```

**Features** are the atomic unit of declared intent. Not tasks. Not tickets. Desired states that are either satisfied or drifting.

**The delta** is the actionable artifact. The gap between declaration and status drives agent work or human decisions. Without a named delta, drift-closing has nowhere to live.

**Session records** are immutable. They capture what the agent read, what it changed, what decisions it surfaced, where it asked for human input. They are not specs — specs described what to do. Session records are evidence of what happened.

**Docs** are derived. Never hand-authored. Generated from declaration + test outcomes + session records. Always current because they cannot be edited directly.

**The capability map** is bidirectional — declared from above by humans, derived from below by the codebase. It is never closed. It is the timeless map of what the system does.

---

## What Ship Does

Ship compiles intent into agent configuration.

It detects installed providers — Claude Code, Gemini CLI, Codex, Cursor, Windsurf — and generates the correct native configuration for each, scoped to the active workspace. Not dotfiles maintained by hand. Structured declarations that produce the right format for the right tool at the right moment.

It enforces security at the runtime level. Permissions are not suggestions to the agent. They are enforced by the Ship runtime before the agent receives its first prompt. Allow/deny patterns, filesystem restrictions, MCP tool filtering — all compiled into every session.

It runs post-session hooks. After every session, tests execute against the feature declaration. Documentation updates. Drift is measured and surfaced — routed to a human if a decision is needed, closed automatically if the system is converging.

It keeps the audit trail. Session records accumulate. The history of what was built, why, and how is permanent and queryable. An agent onboarding to a feature three months later reads the same record a new engineer would.

---

## Agents Are First-Class Users

Ship is not built for developers who use agents. It is built for developers *and* agents — as equal consumers of the same structured intent.

Every interface Ship exposes — the MCP server, the CLI, the compiled workspace context — is designed to be read and acted on by an agent without human translation. An agent does not need to be told where to find the feature declaration, what decisions shaped this module, or which tools are permitted in this workspace. Ship compiles that answer and delivers it before the first prompt.

This is the design constraint that governs every Ship decision. If a human can read it but an agent cannot act on it, it is not finished.

---

## Boundaries

Ship is not a code editor. Agents do the coding.

Ship is not a general project management tool. It has no concept of a ticket unrelated to declared system state.

Ship is not a documentation platform. Docs are a derived output, not a first-class concern.

Ship is not a model. Models are brought by the user.

Ship is not SaaS-first. Local-first is permanent. Cloud sync is additive infrastructure, not the product.

Ship is not trying to replace git. It sits above git, uses git as a transport, and enforces git discipline as a byproduct of its own model.

---

## The Test

**Does this keep the system honest about what it is supposed to be?**

If yes, it belongs in Ship. If it is useful but doesn't close drift, it belongs somewhere else.