-- CLI auth tables for PKCE-based `ship login` flow

-- Ephemeral state created when CLI initiates OAuth (GET /auth/cli).
-- Deleted immediately after the GitHub callback consumes it.
CREATE TABLE IF NOT EXISTS `cli_auth_state` (
    `state`          TEXT    NOT NULL PRIMARY KEY,
    `code_challenge` TEXT    NOT NULL,
    `redirect_uri`   TEXT    NOT NULL,
    `created_at`     INTEGER NOT NULL
);

-- Short-lived auth codes issued by /auth/cli-callback.
-- Consumed exactly once by POST /api/auth/token to issue a JWT.
CREATE TABLE IF NOT EXISTS `cli_auth_codes` (
    `code`           TEXT    NOT NULL PRIMARY KEY,
    `user_id`        TEXT    NOT NULL REFERENCES `user`(`id`),
    `org_id`         TEXT    NOT NULL REFERENCES `orgs`(`id`),
    `code_challenge` TEXT    NOT NULL,
    `created_at`     INTEGER NOT NULL,
    `used`           INTEGER NOT NULL DEFAULT 0
);
