CREATE TABLE `github_installations` (
	`id` text PRIMARY KEY NOT NULL,
	`installation_id` integer NOT NULL,
	`account_login` text NOT NULL,
	`account_type` text NOT NULL,
	`repos_json` text DEFAULT '[]' NOT NULL,
	`created_at` integer NOT NULL,
	`updated_at` integer NOT NULL
);
--> statement-breakpoint
CREATE UNIQUE INDEX `github_installations_installation_id_unique` ON `github_installations` (`installation_id`);--> statement-breakpoint
CREATE INDEX `gh_install_account` ON `github_installations` (`account_login`);--> statement-breakpoint
CREATE TABLE `mcp_servers` (
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
	`status` text DEFAULT 'active' NOT NULL,
	`vetted` integer DEFAULT 0 NOT NULL,
	`synced_at` integer
);
--> statement-breakpoint
CREATE TABLE `package_skills` (
	`id` text PRIMARY KEY NOT NULL,
	`package_id` text NOT NULL,
	`version_id` text NOT NULL,
	`skill_id` text NOT NULL,
	`name` text NOT NULL,
	`description` text,
	`content_hash` text NOT NULL,
	FOREIGN KEY (`package_id`) REFERENCES `packages`(`id`) ON UPDATE no action ON DELETE no action,
	FOREIGN KEY (`version_id`) REFERENCES `package_versions`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE INDEX `pkg_skills_content_hash` ON `package_skills` (`content_hash`);--> statement-breakpoint
CREATE INDEX `pkg_skills_package` ON `package_skills` (`package_id`);--> statement-breakpoint
CREATE INDEX `pkg_skills_package_version` ON `package_skills` (`package_id`,`version_id`);--> statement-breakpoint
CREATE TABLE `package_versions` (
	`id` text PRIMARY KEY NOT NULL,
	`package_id` text NOT NULL,
	`version` text NOT NULL,
	`git_tag` text NOT NULL,
	`commit_sha` text NOT NULL,
	`content_hash` text,
	`skills_json` text,
	`agents_json` text,
	`indexed_at` integer NOT NULL,
	FOREIGN KEY (`package_id`) REFERENCES `packages`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE INDEX `pkg_versions_package_indexed` ON `package_versions` (`package_id`,`indexed_at`);--> statement-breakpoint
CREATE TABLE `packages` (
	`id` text PRIMARY KEY NOT NULL,
	`path` text NOT NULL,
	`scope` text NOT NULL,
	`name` text NOT NULL,
	`description` text,
	`repo_url` text NOT NULL,
	`default_branch` text DEFAULT 'main' NOT NULL,
	`latest_version` text,
	`content_hash` text,
	`source_type` text DEFAULT 'native' NOT NULL,
	`tags` text,
	`claimed_by` text,
	`deprecated_by` text,
	`stars` integer DEFAULT 0 NOT NULL,
	`installs` integer DEFAULT 0 NOT NULL,
	`indexed_at` integer NOT NULL,
	`updated_at` integer NOT NULL
);
--> statement-breakpoint
CREATE UNIQUE INDEX `packages_path_unique` ON `packages` (`path`);--> statement-breakpoint
CREATE INDEX `packages_scope_installs` ON `packages` (`scope`,`installs`);--> statement-breakpoint
CREATE INDEX `packages_path` ON `packages` (`path`);--> statement-breakpoint
CREATE INDEX `packages_name` ON `packages` (`name`);--> statement-breakpoint
CREATE INDEX `packages_description` ON `packages` (`description`);--> statement-breakpoint
CREATE INDEX `packages_claimed_by` ON `packages` (`claimed_by`);