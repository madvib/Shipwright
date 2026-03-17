-- Better Auth tables (https://www.better-auth.com/docs/concepts/database)
-- SQLite/D1 schema — camelCase column names match better-auth defaults

CREATE TABLE `user` (
    `id`            TEXT    NOT NULL PRIMARY KEY,
    `name`          TEXT    NOT NULL,
    `email`         TEXT    NOT NULL UNIQUE,
    `emailVerified` INTEGER NOT NULL,
    `image`         TEXT,
    `createdAt`     INTEGER NOT NULL,
    `updatedAt`     INTEGER NOT NULL
);

CREATE TABLE `session` (
    `id`        TEXT    NOT NULL PRIMARY KEY,
    `expiresAt` INTEGER NOT NULL,
    `token`     TEXT    NOT NULL UNIQUE,
    `createdAt` INTEGER NOT NULL,
    `updatedAt` INTEGER NOT NULL,
    `ipAddress` TEXT,
    `userAgent` TEXT,
    `userId`    TEXT    NOT NULL REFERENCES `user`(`id`)
);

CREATE TABLE `account` (
    `id`                     TEXT    NOT NULL PRIMARY KEY,
    `accountId`              TEXT    NOT NULL,
    `providerId`             TEXT    NOT NULL,
    `userId`                 TEXT    NOT NULL REFERENCES `user`(`id`),
    `accessToken`            TEXT,
    `refreshToken`           TEXT,
    `idToken`                TEXT,
    `accessTokenExpiresAt`   INTEGER,
    `refreshTokenExpiresAt`  INTEGER,
    `scope`                  TEXT,
    `password`               TEXT,
    `createdAt`              INTEGER NOT NULL,
    `updatedAt`              INTEGER NOT NULL
);

CREATE TABLE `verification` (
    `id`         TEXT    NOT NULL PRIMARY KEY,
    `identifier` TEXT    NOT NULL,
    `value`      TEXT    NOT NULL,
    `expiresAt`  INTEGER NOT NULL,
    `createdAt`  INTEGER,
    `updatedAt`  INTEGER
);
