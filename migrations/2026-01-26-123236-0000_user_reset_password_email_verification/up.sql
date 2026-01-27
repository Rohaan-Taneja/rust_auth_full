-- table to verify existing user(non logged in) email , who wants to reset his password 

CREATE TABLE "user_reset_password_email_verifications" (
    id UUID PRIMARY KEY DEFAULT uuid_generate_V4(),
    user_email VARCHAR(255) NOT NULL REFERENCES users(email) ON DELETE CASCADE,
    otp VARCHAR(6) NOT NULL,
    expires_at TIMESTAMP with TIME ZONE ,
    used BOOLEAN NOT NULL DEFAULT FALSE ,
    created_at TIMESTAMP with TIME ZONE DEFAULT NOW()


);

CREATE INDEX user_reset_pass_email_verification_idx on user_reset_password_email_verifications (user_email);
