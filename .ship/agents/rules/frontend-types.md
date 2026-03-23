All frontend types must be derived from a data source — never invented for UI convenience.

- Import types from `@ship/ui` (generated from Rust compiler via Specta)
- Import types from `#/db/schema` (Drizzle D1 schema) for server-side data
- API response types derive from schema types, never standalone interfaces
- If the UI needs a resolved/denormalized view, create it as a computed type using Pick/Omit/intersection of real types — not a parallel interface
- Component props that describe data shape must reference the source type
- UI-only state (dialog open, scroll position, active tab) is the only legitimate local type
- No hardcoded data arrays in components — data comes from props, hooks, or API
- No silent no-op onClick handlers — every button does something observable or is disabled with a reason
- `@ship/primitives` for shared UI components (shadcn), `@ship/ui` for domain editors and generated types
