-- This file should undo anything in `up.sql`

DROP INDEX IF EXISTS user_reset_pass_email_verification_idx;

DROP TABLE IF EXISTS "user_reset_password_email_verifications";