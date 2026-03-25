# Working With Agents

Hard-won lessons from building Ship with AI agents every day.

## Human attention is the most valuable resource

An agent can generate 1,000 lines in a minute. You can review maybe 100 lines with full comprehension in that same minute. The bottleneck is never code generation — it's understanding what was generated. Optimize for your attention, not the agent's throughput.

## A session is an entire career in 45 minutes

Every agent session follows the same arc: onboard, execute, exit.

**Onboard.** Give the agent the context it needs — specs, file paths, constraints, what NOT to do. Skimping here produces confident garbage. The agent has never seen your codebase before. Every time.

**Execute.** Let it work. Interrupt for course corrections, not commentary.

**Exit interview.** Extract the value: what changed, what decisions were made, what's unfinished. Discard the noise: reasoning chains, hedging, status updates nobody will read. If you don't capture the output, the session may as well not have happened.

## Context rot is the root of all evil

Stale context produces stale output. An agent working from yesterday's plan against today's code will build the wrong thing with total confidence.

Good in, good out. There is no shortcut. An organized approach — current specs, clean file scope, verified assumptions — is the only way to get reliable results from agents. Without it you are creating more work for yourself.

## Code is cheap

It is often cheaper to discard agent output and rebuild than to debug something you don't fully understand. The sunk cost fallacy hits harder with agents because they produce so much so fast. Resist it. If you can't explain why the code works, throw it away.

## The illusion of progress is worse than no progress

Agents are exceptionally good at producing things that look finished. A polished UI with no backend. A test suite that passes but tests nothing. Copy that reads well but says nothing. Buttons that exist but don't work.

This is two-dimensional software. It demos well. It ships badly. What happens is that components an agent added to create a convincing demo become vestigial fragments that subsequent agents interpret as intentional parts of the design. The scaffold becomes load-bearing.

The antidote: verify everything. Click every button. Read every test assertion. Trace every data path. If you wouldn't ship it without an agent, don't ship it with one.

## Don't trust. Verify.

Agents come across issues and move on. They solve the problem in front of them and leave adjacent problems for someone else — except there is no someone else. The issues accumulate silently until they surface as bugs in production or architectural debt that blocks the next feature.

The fix is cultural, not technical. Treat agent output like a pull request from a new hire: assume competence, verify everything, and never merge what you haven't read.
