-- Org model — multi-tenant tables for Ship cloud product
-- Adds orgs, org-scoped users, workspaces, and agent sessions

CREATE TABLE IF NOT EXISTS `orgs` (
    `id`         TEXT    NOT NULL PRIMARY KEY,
    `name`       TEXT    NOT NULL,
    `slug`       TEXT    NOT NULL UNIQUE,
    `created_at` INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS `org_members` (
    `id`         TEXT    NOT NULL PRIMARY KEY,
    `org_id`     TEXT    NOT NULL REFERENCES `orgs`(`id`),
    `user_id`    TEXT    NOT NULL REFERENCES `user`(`id`),
    `role`       TEXT    NOT NULL DEFAULT 'member',
    `created_at` INTEGER NOT NULL,
    UNIQUE (`org_id`, `user_id`)
);

CREATE TABLE IF NOT EXISTS `workspaces` (
    `id`         TEXT    NOT NULL PRIMARY KEY,
    `org_id`     TEXT    NOT NULL REFERENCES `orgs`(`id`),
    `name`       TEXT    NOT NULL,
    `branch`     TEXT    NOT NULL DEFAULT 'main',
    `status`     TEXT    NOT NULL DEFAULT 'idle',
    `created_at` INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS `agent_sessions` (
    `id`           TEXT    NOT NULL PRIMARY KEY,
    `workspace_id` TEXT    NOT NULL REFERENCES `workspaces`(`id`),
    `provider`     TEXT    NOT NULL,
    `started_at`   INTEGER NOT NULL,
    `ended_at`     INTEGER
);
