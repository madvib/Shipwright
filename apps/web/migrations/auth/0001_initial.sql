-- ship-auth: initial schema
-- Better Auth tables (user, session, account, verification) + CLI auth tables

CREATE TABLE IF NOT EXISTS `user` (
  `id` text PRIMARY KEY NOT NULL,
  `name` text NOT NULL,
  `email` text NOT NULL,
  `emailVerified` integer NOT NULL,
  `image` text,
  `createdAt` integer NOT NULL,
  `updatedAt` integer NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS `user_email_unique` ON `user` (`email`);

CREATE TABLE IF NOT EXISTS `session` (
  `id` text PRIMARY KEY NOT NULL,
  `expiresAt` integer NOT NULL,
  `token` text NOT NULL,
  `createdAt` integer NOT NULL,
  `updatedAt` integer NOT NULL,
  `ipAddress` text,
  `userAgent` text,
  `userId` text NOT NULL REFERENCES `user`(`id`)
);

CREATE UNIQUE INDEX IF NOT EXISTS `session_token_unique` ON `session` (`token`);

CREATE TABLE IF NOT EXISTS `account` (
  `id` text PRIMARY KEY NOT NULL,
  `accountId` text NOT NULL,
  `providerId` text NOT NULL,
  `userId` text NOT NULL REFERENCES `user`(`id`),
  `accessToken` text,
  `refreshToken` text,
  `idToken` text,
  `accessTokenExpiresAt` integer,
  `refreshTokenExpiresAt` integer,
  `scope` text,
  `password` text,
  `createdAt` integer NOT NULL,
  `updatedAt` integer NOT NULL
);

CREATE TABLE IF NOT EXISTS `verification` (
  `id` text PRIMARY KEY NOT NULL,
  `identifier` text NOT NULL,
  `value` text NOT NULL,
  `expiresAt` integer NOT NULL,
  `createdAt` integer,
  `updatedAt` integer
);

CREATE TABLE IF NOT EXISTS `cli_auth_state` (
  `state` text PRIMARY KEY NOT NULL,
  `code_challenge` text NOT NULL,
  `redirect_uri` text NOT NULL,
  `created_at` integer NOT NULL
);

CREATE TABLE IF NOT EXISTS `cli_auth_codes` (
  `code` text PRIMARY KEY NOT NULL,
  `user_id` text NOT NULL,
  `code_challenge` text NOT NULL,
  `created_at` integer NOT NULL,
  `used` integer NOT NULL DEFAULT 0
);
