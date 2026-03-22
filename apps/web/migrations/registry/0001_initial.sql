-- ship-registry: initial schema
-- packages, versions, skills, github installations, mcp servers

CREATE TABLE IF NOT EXISTS `packages` (
  `id` text PRIMARY KEY NOT NULL,
  `path` text NOT NULL,
  `scope` text NOT NULL,
  `name` text NOT NULL,
  `description` text,
  `repo_url` text NOT NULL,
  `default_branch` text NOT NULL DEFAULT 'main',
  `latest_version` text,
  `content_hash` text,
  `source_type` text NOT NULL DEFAULT 'native',
  `tags` text,
  `claimed_by` text,
  `deprecated_by` text,
  `stars` integer NOT NULL DEFAULT 0,
  `installs` integer NOT NULL DEFAULT 0,
  `indexed_at` integer NOT NULL,
  `updated_at` integer NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS `packages_path_unique` ON `packages` (`path`);
CREATE INDEX IF NOT EXISTS `packages_scope_installs` ON `packages` (`scope`, `installs`);
CREATE INDEX IF NOT EXISTS `packages_path` ON `packages` (`path`);
CREATE INDEX IF NOT EXISTS `packages_name` ON `packages` (`name`);
CREATE INDEX IF NOT EXISTS `packages_description` ON `packages` (`description`);
CREATE INDEX IF NOT EXISTS `packages_claimed_by` ON `packages` (`claimed_by`);

CREATE TABLE IF NOT EXISTS `package_versions` (
  `id` text PRIMARY KEY NOT NULL,
  `package_id` text NOT NULL REFERENCES `packages`(`id`),
  `version` text NOT NULL,
  `git_tag` text NOT NULL,
  `commit_sha` text NOT NULL,
  `content_hash` text,
  `skills_json` text,
  `agents_json` text,
  `indexed_at` integer NOT NULL
);

CREATE INDEX IF NOT EXISTS `pkg_versions_package_indexed` ON `package_versions` (`package_id`, `indexed_at`);

CREATE TABLE IF NOT EXISTS `package_skills` (
  `id` text PRIMARY KEY NOT NULL,
  `package_id` text NOT NULL REFERENCES `packages`(`id`),
  `version_id` text NOT NULL REFERENCES `package_versions`(`id`),
  `skill_id` text NOT NULL,
  `name` text NOT NULL,
  `description` text,
  `content_hash` text NOT NULL
);

CREATE INDEX IF NOT EXISTS `pkg_skills_content_hash` ON `package_skills` (`content_hash`);
CREATE INDEX IF NOT EXISTS `pkg_skills_package` ON `package_skills` (`package_id`);
CREATE INDEX IF NOT EXISTS `pkg_skills_package_version` ON `package_skills` (`package_id`, `version_id`);

CREATE TABLE IF NOT EXISTS `github_installations` (
  `id` text PRIMARY KEY NOT NULL,
  `installation_id` integer NOT NULL,
  `account_login` text NOT NULL,
  `account_type` text NOT NULL,
  `repos_json` text NOT NULL DEFAULT '[]',
  `created_at` integer NOT NULL,
  `updated_at` integer NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS `github_installations_installation_id_unique` ON `github_installations` (`installation_id`);
CREATE INDEX IF NOT EXISTS `gh_install_account` ON `github_installations` (`account_login`);

CREATE TABLE IF NOT EXISTS `mcp_servers` (
  `name` text PRIMARY KEY NOT NULL,
  `title` text,
  `description` text,
  `homepage` text,
  `vendor` text,
  `tags` text,
  `package_registry` text,
  `command` text,
  `args` text,
  `image_url` text,
  `status` text NOT NULL DEFAULT 'active',
  `vetted` integer NOT NULL DEFAULT 0,
  `synced_at` integer
);
