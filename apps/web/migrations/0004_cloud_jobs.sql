-- Cloud job queue — agent jobs dispatched via the Ship API

CREATE TABLE IF NOT EXISTS `cloud_jobs` (
    `id`           TEXT    NOT NULL PRIMARY KEY,
    `org_id`       TEXT    NOT NULL REFERENCES `orgs`(`id`),
    `workspace_id` TEXT    REFERENCES `workspaces`(`id`),
    `type`         TEXT    NOT NULL,
    `status`       TEXT    NOT NULL DEFAULT 'pending',
    `payload`      TEXT,
    `created_at`   INTEGER NOT NULL,
    `updated_at`   INTEGER NOT NULL
);
