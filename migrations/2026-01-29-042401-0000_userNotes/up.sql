-- userNotes

CREATE TABLE "user_notes" (
    id UUID PRIMARY KEY DEFAULT uuid_generate_V4() ,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL ,
    content VARCHAR(1000) NOT NULL,
    created_at TIMESTAMP with TIME ZONE DEFAULT NOW()
 );

CREATE INDEX user_notes_idx ON user_notes (user_id);