+++
id = "6a3848fa-7c0e-4066-9e9c-0b45cfeb50b2"
title = "MCP Sampling for UI Generation"
created = "2026-02-22T05:30:30.844675822Z"
updated = "2026-02-23T18:21:51.729332Z"
tags = []
links = []
+++

Use MCP sampling to allow AI-assisted generation directly inside the UI, enabling in-context completions and suggestions without leaving the app.

Wired MCP-sample affordance into markdown editing surfaces (Issue/ADR/Spec editors and create flows) with defensive fallback template generation while runtime MCP chat sampling API is still pending. Next step is replacing template fallback with real scoped sampling command.

Sampling UX hardened: sample insertion is now non-destructive (appends below existing content instead of overwrite) and includes one-click Undo Sample. Button copy updated to reflect insertion semantics; this protects user content while true MCP-backed generation is wired.
