-- libraries: user-owned config snapshots
CREATE TABLE IF NOT EXISTS `libraries` (
  `id` TEXT PRIMARY KEY,
  `org_id` TEXT NOT NULL REFERENCES `orgs`(`id`),
  `user_id` TEXT NOT NULL REFERENCES `user`(`id`),
  `name` TEXT NOT NULL,
  `slug` TEXT,
  `data` TEXT NOT NULL DEFAULT '{}',
  `created_at` INTEGER NOT NULL,
  `updated_at` INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS `libraries_org_user` ON `libraries`(`org_id`, `user_id`);
CREATE UNIQUE INDEX IF NOT EXISTS `libraries_org_slug` ON `libraries`(`org_id`, `slug`) WHERE `slug` IS NOT NULL;

-- profiles: compiled agent profiles
CREATE TABLE IF NOT EXISTS `profiles` (
  `id` TEXT PRIMARY KEY,
  `org_id` TEXT NOT NULL REFERENCES `orgs`(`id`),
  `user_id` TEXT NOT NULL REFERENCES `user`(`id`),
  `name` TEXT NOT NULL,
  `content` TEXT NOT NULL,
  `provider` TEXT,
  `created_at` INTEGER NOT NULL,
  `updated_at` INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS `profiles_org_user` ON `profiles`(`org_id`, `user_id`);

-- workflows: visual workflow definitions
CREATE TABLE IF NOT EXISTS `workflows` (
  `id` TEXT PRIMARY KEY,
  `org_id` TEXT NOT NULL REFERENCES `orgs`(`id`),
  `user_id` TEXT NOT NULL REFERENCES `user`(`id`),
  `name` TEXT NOT NULL,
  `definition` TEXT NOT NULL DEFAULT '{}',
  `created_at` INTEGER NOT NULL,
  `updated_at` INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS `workflows_org_user` ON `workflows`(`org_id`, `user_id`);
