
ALTER TABLE user_reset_pass_validations
ADD COLUMN created_at TIMESTAMPTZ DEFAULT NOW();
