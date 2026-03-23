A frontend type that does not derive from a data source is a lie. Do not lie.

Types come from exactly two places:
- `@ship/ui` — generated from the Rust compiler via Specta. This is the source of truth for all agent, compiler, and config types. Import them directly in the components that use them. Do not re-export through barrel files.
- `#/db/schema` — Drizzle D1 schema. This is the source of truth for all server-side data (packages, profiles, workflows).

Any type that duplicates, shadows, or "extends" a generated type with invented fields is a lie. It will drift from the source, create a translation layer, and the next agent will treat the lie as intentional architecture.

Rules:
- Import `@ship/ui` types directly where needed. No local `types.ts` barrel files that re-export or wrap them.
- API response types derive from schema types, never standalone interfaces.
- Use TanStack Start `createServerFn` for end-to-end type safety on server functions. Do not use untyped `fetch` + `Response.json()`.
- If the UI needs a resolved/joined view (e.g., skill refs resolved to full Skill objects), express it as a computed type using Pick/Omit/intersection of real types — not a parallel interface.
- UI-only state (dialog open, scroll position, active tab) is the only legitimate local type definition.
- No hardcoded data arrays in components. Data comes from props, hooks, or API. A `const ITEMS = [...]` in a component file is a lie unless it is a fixed UI constant (like nav items or icon mappings).
- No silent no-op `onClick` handlers. Every interactive element does something observable or is visually disabled with a reason. A button that does nothing is a lie.
- No `OrangeDot` or similar "not implemented" indicators that ship to users. Either implement it or remove it.
- `@ship/primitives` for shared UI building blocks (shadcn). `@ship/ui` for generated types. Import directly, do not create local wrappers.
