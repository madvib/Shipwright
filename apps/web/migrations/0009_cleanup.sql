-- Cleanup: drop dead tables from early migrations
--
-- Evidence (grep -r across apps/web/src/):
--   workspace (singular, 0002): 0 references — superseded by workspaces (plural, 0003)
--   project (0002):             0 references — never used
--   agent_sessions (0003):      0 references — never used
--
-- Audited 2026-03-21.

DROP TABLE IF EXISTS `project`;
DROP TABLE IF EXISTS `workspace`;
DROP TABLE IF EXISTS `agent_sessions`;
