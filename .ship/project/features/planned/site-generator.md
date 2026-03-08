<!-- ship:feature id=WYuFL87v -->

# Site Generator

## Why

Ship should support publishing project intelligence (feature docs, decisions, release narratives) as structured external documentation without manual copy workflows.

## Acceptance Criteria

- [ ] Generator can produce docs output from canonical project records
- [ ] Output profiles support internal docs and public-facing release/docs sites
- [ ] Build process supports stable identifiers and incremental regeneration
- [ ] Generated output can be deployed to standard static hosts

## Delivery Todos

- [ ] Define generator input model from Ship canonical data
- [ ] Implement baseline static output profile
- [ ] Add deploy-ready adapters/templates for common hosts
- [ ] Add link integrity/versioning checks in generation pipeline

## Current Behavior

Planned capability; no generator pipeline is shipped in alpha.

## Follow-ups

- Coordinate with docs data model evolution and future cloud publish flow.