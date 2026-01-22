
ALTER TABLE users
  ALTER COLUMN verification_token DROP NOT NULL,
  ALTER COLUMN token_expires_at DROP NOT NULL;