-- table to verify user email 

CREATE TABLE "user_email_verifications" (
    id UUID PRIMARY KEY DEFAULT uuid_generate_V4(),
    user_email VARCHAR(255) NOT NULL REFERENCES users(email) ON DELETE CASCADE,
    otp VARCHAR(6) NOT NULL,
    expires_at TIMESTAMP with TIME ZONE ,
    used BOOLEAN NOT NULL DEFAULT FALSE


);

CREATE INDEX idx_email_verification_user_id on user_email_verifications (user_email);
