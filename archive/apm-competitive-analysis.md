# APM Competitive Analysis — Ship vs Microsoft APM

Date: 2026-03-19

## Where APM beats us today

**1. The CLI works.**
APM is at v0.8.2 with a complete, documented CLI. Ship's CLI is incomplete — `ship use` and `ship init` work, but `ship install`, `ship add`, and dep resolution are not wired end-to-end. Six blocking jobs in cli-v0.1-capabilities.json must land before we can credibly claim to be a package manager.

**2. Security auditing.**
APM's `apm audit` scans for hidden Unicode attacks (bidi marks, variation selectors, invisible math operators) and outputs SARIF for CI/CD gates. Ship has nothing equivalent. This is the feature that gets past enterprise security reviews.

**3. Transitive dependency resolution.**
APM resolves full dep trees — packages that depend on packages, lockfiles, cycle detection, version constraints. Ship's registry is git-native but v0.1 only handles direct deps.

**4. Enterprise distribution.**
Homebrew, Scoop, pip, native binaries, install scripts, Artifactory proxy, air-gapped mode. APM can land in any corporate environment.

**5. GitHub Copilot integration.**
Microsoft owns GitHub. APM + Copilot is their home court. Don't fight there.

---

## Where Ship beats APM (the moat)

**1. Provider settings depth.**
Ship covers ~90% of Claude Code native settings, ~80% Gemini, ~70% Codex, ~85% Cursor.
APM covers ~40% of Claude (hooks only), essentially nothing for Gemini/Codex/Cursor settings.
Concrete, measurable, demonstrable.

**2. The compiler is real.**
APM's "compilation" is markdown aggregation and file copying.
Ship's Rust/WASM compiler is a schema-stable transform — `ProjectLibrary → CompileOutput`.
Runs in the browser, CLI, anywhere WASM runs. APM cannot replicate without a rewrite.

**3. Ship is runtime infrastructure. APM is a file deployer.**
APM writes files and stops. Ship's MCP server provides 30+ tools for agent coordination —
workspaces, sessions, job queues, file ownership, ADRs. Completely different product tier.

**4. Branch-aware config.**
No one else has this. Switch branches, agent switches context automatically.
Solves a real pain point every team using agents across feature branches feels.

**5. Studio.**
Zero install friction. Paste a GitHub URL, compile for any provider, download.
APM requires CLI install. Ship Studio makes the value immediate.

---

## What to build to close the gap and win

### Tier 1 — Ship these or stay a prototype

| What | Why |
|---|---|
| Finish the 6 CLI v0.1 capabilities | Can't call ourselves a package manager with broken `ship install` |
| `ship audit` — content security scanning | Blocks enterprise without it. SARIF output for CI/CD |
| Registry end-to-end | `ship add owner/repo`, `ship install`, lockfile update |
| Install script + Homebrew tap | Distribution is moat |

### Tier 2 — Differentiation that compounds

| What | Why |
|---|---|
| Branch-aware auto-switch (post-checkout hook) | Most unique feature — ship it |
| `ship matrix` exposed in Studio | Show users exactly what Ship compiles vs provider support — APM has nothing like this |
| Transitive dep resolution | Table stakes for a package manager |
| CI/CD mode (`ship compile --ci --format sarif`) | Gets Ship into pipelines alongside APM |

### Tier 3 — Documentation and positioning

Write the comparison explicitly. Make the provider coverage matrix public.
Show the numbers — 90% Claude Code settings coverage vs APM's 40%.

---

## Positioning bet

APM wins the GitHub Copilot market. Let them.

Our market: everyone using Claude, Gemini, Codex, and Cursor who wants their agent to know
what branch they're on, what tools it's allowed to use, and what workspace it's in.
APM doesn't solve that problem. Ship does.

**The risk:** APM is simpler and Microsoft has distribution. If teams just want to share a
CLAUDE.md, APM is easier. Ship needs a working CLI before it can compete on that tier at all.

**Ship the CLI.**
