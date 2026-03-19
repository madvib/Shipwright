-- Registry tables: packages, versions, and skills index
-- Packages are git-hosted; D1 stores metadata only.

CREATE TABLE IF NOT EXISTS `packages` (
  `id` TEXT PRIMARY KEY NOT NULL,
  `path` TEXT NOT NULL UNIQUE,
  `scope` TEXT NOT NULL DEFAULT 'community',
  `name` TEXT NOT NULL,
  `description` TEXT,
  `repo_url` TEXT NOT NULL,
  `default_branch` TEXT NOT NULL DEFAULT 'main',
  `latest_version` TEXT,
  `content_hash` TEXT,
  `source_type` TEXT NOT NULL DEFAULT 'native',
  `claimed_by` TEXT REFERENCES `user`(`id`),
  `deprecated_by` TEXT,
  `stars` INTEGER NOT NULL DEFAULT 0,
  `installs` INTEGER NOT NULL DEFAULT 0,
  `indexed_at` INTEGER NOT NULL,
  `updated_at` INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS `packages_scope_installs` ON `packages` (`scope`, `installs`);
CREATE INDEX IF NOT EXISTS `packages_path` ON `packages` (`path`);

CREATE TABLE IF NOT EXISTS `package_versions` (
  `id` TEXT PRIMARY KEY NOT NULL,
  `package_id` TEXT NOT NULL REFERENCES `packages`(`id`),
  `version` TEXT NOT NULL,
  `git_tag` TEXT NOT NULL,
  `commit_sha` TEXT NOT NULL,
  `content_hash` TEXT,
  `skills_json` TEXT,
  `agents_json` TEXT,
  `indexed_at` INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS `pkg_versions_package_indexed` ON `package_versions` (`package_id`, `indexed_at`);

CREATE TABLE IF NOT EXISTS `package_skills` (
  `id` TEXT PRIMARY KEY NOT NULL,
  `package_id` TEXT NOT NULL REFERENCES `packages`(`id`),
  `version_id` TEXT NOT NULL REFERENCES `package_versions`(`id`),
  `skill_id` TEXT NOT NULL,
  `name` TEXT NOT NULL,
  `description` TEXT,
  `content_hash` TEXT NOT NULL,
  `content_length` INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS `pkg_skills_content_hash` ON `package_skills` (`content_hash`);
CREATE INDEX IF NOT EXISTS `pkg_skills_package` ON `package_skills` (`package_id`);
