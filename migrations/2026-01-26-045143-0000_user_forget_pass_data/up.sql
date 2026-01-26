-- THIS SCHEMA WILL TEMP STORE HASHED TOKEN , WHEN NON LOGGEN USER WANT TO CHANGE HIS PASS , THIS WILL VERIFY , THAT THE USER IS VERIFIED/ORGINAL
CREATE TABLE "user_reset_pass_validations" (
    id UUID PRIMARY KEY DEFAULT uuid_generate_V4(),
    user_email VARCHAR(255) not NULL REFERENCES users(email) ON DELETE CASCADE,
    hashed_reset_token VARCHAR(100) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE,
    used BOOLEAN NOT NULL DEFAULT FALSE
);

-- fast searching via user email
CREATE INDEX user_reset_pass_validation_idx on user_reset_pass_validations (user_email);