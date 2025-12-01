-- This file should undo anything in `up.sql`

-- Reverse of CREATE INDEX
DROP INDEX IF EXISTS user_email_idx;

-- Reverse of CREATE TABLE
DROP TABLE IF EXISTS "users";

-- Reverse of CREATE TYPE
DROP TYPE IF EXISTS USER_TYPE;

-- We do NOT drop extension "uuid-ossp"
-- because extensions are usually managed globally,
-- not per migration, so Diesel migrations typically don't remove them.
