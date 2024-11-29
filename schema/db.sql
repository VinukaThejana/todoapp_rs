CREATE EXTENSION IF NOT EXISTS ulid;

CREATE
OR REPLACE FUNCTION get_epoch() RETURNS BIGINT AS
$$
BEGIN
RETURN EXTRACT(
    EPOCH
    FROM
        NOW()
) :: BIGINT;

END;

$$
LANGUAGE 'plpgsql';

CREATE
OR REPLACE FUNCTION updated_at() RETURNS TRIGGER AS
$$
BEGIN
NEW.updated_at = get_epoch();

RETURN NEW;

END;

$$
LANGUAGE 'plpgsql';

CREATE TABLE IF NOT EXISTS user_ (
    "id" VARCHAR(26) PRIMARY KEY DEFAULT gen_ulid(),
    "name" VARCHAR(255) NOT NULL,
    "email" VARCHAR(255) NOT NULL UNIQUE,
    "password" VARCHAR(255) NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS "idx_user_email_" ON "user_" ("email");

CREATE TABLE IF NOT EXISTS "session_" (
    "id" VARCHAR(26) PRIMARY KEY,
    "user_id" VARCHAR(26) NOT NULL,
    "expires" BIGINT NOT NULL,
    "login_at" BIGINT NOT NULL DEFAULT get_epoch(),
    CONSTRAINT "fk_session_user_id_" FOREIGN KEY ("user_id") REFERENCES "user_" ("id") ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS "idx_session_user_id_" ON "session_" ("user_id");

CREATE INDEX IF NOT EXISTS "idx_session_expires_" ON "session_" ("expires");

CREATE TABLE IF NOT EXISTS "todo_" (
    "id" VARCHAR(26) PRIMARY KEY DEFAULT gen_ulid(),
    "user_id" VARCHAR(26) NOT NULL,
    "title" VARCHAR(255) NOT NULL,
    "content" TEXT NOT NULL,
    "completed" BOOLEAN NOT NULL DEFAULT FALSE,
    "created_at" BIGINT NOT NULL DEFAULT get_epoch(),
    "updated_at" BIGINT NOT NULL DEFAULT get_epoch(),
    CONSTRAINT "fk_todo_user_id_" FOREIGN KEY ("user_id") REFERENCES "user_" ("id") ON DELETE CASCADE
);

DROP TRIGGER IF EXISTS "todo_updated_at_" ON "todo_";

CREATE TRIGGER "todo_updated_at_" BEFORE
UPDATE
    ON "todo_" FOR EACH ROW EXECUTE FUNCTION updated_at();
