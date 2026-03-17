-- App tables — minimal, expandable

CREATE TABLE `workspace` (
    `id`        TEXT    NOT NULL PRIMARY KEY,
    `name`      TEXT    NOT NULL,
    `userId`    TEXT    NOT NULL REFERENCES `user`(`id`),
    `createdAt` INTEGER NOT NULL,
    `updatedAt` INTEGER NOT NULL
);

CREATE TABLE `project` (
    `id`          TEXT    NOT NULL PRIMARY KEY,
    `name`        TEXT    NOT NULL,
    `workspaceId` TEXT    NOT NULL REFERENCES `workspace`(`id`),
    `createdAt`   INTEGER NOT NULL,
    `updatedAt`   INTEGER NOT NULL
);
